use sha2::{Sha512, Digest};
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, VecDeque};
use std::iter;
use std::iter::{empty, from_fn, FromIterator};
use std::mem::size_of;
use anyhow::anyhow;
use smallvec::SmallVec;
use crate::config::config;
use crate::{either_gnote};
use crate::gnote::Gnote;
use crate::simple_note;
use crate::measure::{Measure, MeasureNumberType};
use crate::pitch::PsType;
use crate::simple_note::{SimpleNote, TieInfo};
use super::attribs::*;

#[derive(Clone)]
pub struct Part {
    pub name: String,
    pub key_sig: KeySignature,
    pub clef_sign: ClefType,
    pub time_sig: TimeSig,
    pub gnotes: Vec<Gnote>
}

impl Part {
    pub fn new(
        part_name: String,
        key_sig: KeySignature,
        clef_sign: ClefType,
        time_sig: TimeSig
    ) -> Part
    {
        Part {
            name: part_name,
            key_sig,
            clef_sign,
            time_sig,
            gnotes: Vec::new()
        }
    }

    pub fn to_measured(&self) -> MeasuredPart
    {
        let mut measured_part = MeasuredPart::new(
            self.name.clone(),
            self.key_sig,
            self.clef_sign,
            self.time_sig
        );
        if self.gnotes.is_empty() { return measured_part; }

        // Measure length in quarter notes of time signature a/b
        // is given by a * (4 / b)
        let measure_length =
            Duration::new((self.time_sig.numer() * TimeSigComponent::from(4)) as BeatDivision,
                          *self.time_sig.denom() as BeatDivision);
        let est_number_of_measures =
            1 +
            (either_gnote!(self.gnotes.last().unwrap(), gn => gn.interval.end) / measure_length)
                .to_integer();
        measured_part.measures.reserve(est_number_of_measures as usize);
        // initial empty measure
        measured_part.measures.push(
            Measure::new(
                Offset::new(0, 1),
                measure_length,
                0 as MeasureNumberType,
                Vec::new()
            )
        );

        // TODO: NOT SURE IF WORKS, MUST TEST
        let mut _gnotes = RefCell::new(VecDeque::from_iter(self.gnotes.clone()));
        while let Some(cur_gnote) = _gnotes.borrow_mut().pop_front()
        {
            let current_measure_window
                = measured_part
                .measures
                .last()
                .unwrap()
                .interval;

            if current_measure_window
                .does_swallow(either_gnote!(&cur_gnote, gn => gn.interval).borrow())
            {
                let shifted_gnote = cur_gnote;
                either_gnote!(&shifted_gnote, gn => gn.interval)
                    .displace_start_keep_length(-current_measure_window.start);

                measured_part
                    .measures
                    .last_mut()
                    .unwrap()
                    .gnotes
                    .push(shifted_gnote);
            }
            else if current_measure_window.end
                == either_gnote!(&cur_gnote, gn => gn.interval.start)
            {
                // introduce new measure
                measured_part.measures.push(
                    Measure::new(
                        current_measure_window.end,
                        measure_length,
                        measured_part
                            .measures
                            .last()
                            .unwrap()
                            .measure_number + 1,
                        Vec::new()
                    )
                );

                let new_current_measure = measured_part.measures.last_mut().unwrap();

                let shifted_gnote = cur_gnote;
                either_gnote!(&shifted_gnote, gn => gn.interval)
                    .displace_start_keep_length(-new_current_measure.interval.start);

                new_current_measure.gnotes.push(shifted_gnote);
            }
            else if current_measure_window
                .does_overlap_with(either_gnote!(&cur_gnote, gn => &gn.interval))
            {
                let (mut first_half, mut second_half)
                    = either_gnote!(cur_gnote, gn => gn.split_at_offset(current_measure_window.end));

                // Shift offset
                [&mut first_half, &mut second_half]
                .iter_mut()
                .flat_map(|half| {half.iter_mut()})
                .for_each(
                    |snote| {
                        snote.interval.displace_start_keep_length(-current_measure_window.start)
                    }
                );

                measured_part
                    .measures
                    .last_mut()
                    .unwrap()
                    .gnotes
                    .extend(
                        first_half
                        .into_iter()
                        .map( |sn| { Gnote::from(sn) } )
                    );

                let current_measure_number
                = measured_part
                    .measures
                    .last()
                    .unwrap()
                    .measure_number;

                measured_part
                    .measures
                    .push(
                        Measure::new(
                            current_measure_window.end,
                            measure_length,
                            current_measure_number + 1,
                            Vec::new()
                        )
                    );

                // surgery to retains invariant
                {
                    let mut mut_gnotes = _gnotes.borrow_mut();
                    mut_gnotes.reserve(second_half.len());
                    second_half
                        .into_iter()
                        .rev()
                        .map(|sn| { Gnote::SimpleNote(sn) })
                        .for_each(|gn| { mut_gnotes.push_front(gn) })
                }
            }
            else {unreachable!()}
        }
        assert!(
            measured_part
            .measures
            .iter()
            .all( |mea| {mea.get_elements_acc_duration() == measure_length} )
        );

        measured_part
    }

    fn fuse_tied_notes_in_range(note_range: &[simple_note::SimpleNote])
        -> anyhow::Result<Vec<simple_note::SimpleNote>>
    {
        let mut note_stream = Vec::<SimpleNote>::new();
        for cur_note in note_range.to_vec() {
            if cur_note.tie_info == TieInfo::TieNeither {
                note_stream.push(cur_note);
            }
            else {
                if cur_note.tie_info.intersects(TieInfo::TieEnd) {
                    // If potential note to be joined with exists
                    if let Some(potential_tie_origin) = note_stream.last_mut() {
                        // If potential note to be joined exhibit TieStart
                        if potential_tie_origin.tie_info.intersects(TieInfo::TieStart) {
                            // Check if notes are joinable
                            if potential_tie_origin.interval.end == cur_note.interval.start &&
                                potential_tie_origin
                                .pitches
                                .iter()
                                .eq_by(cur_note.pitches.iter(),
                                       |x, y|{x.ps == y.ps})
                            {
                                potential_tie_origin.interval.merge_with(&cur_note.interval);

                                // Combine ties because tie_origin has TieStart
                                // which cancels with TieEnd in cur_note
                                // (turn off TieStart in potential_tie_origin)
                                potential_tie_origin.tie_info &=  (TieInfo::TieStart.complement());
                                assert!(!potential_tie_origin.tie_info.intersects(TieInfo::TieStart));

                                // If the cur_note contains TieStart, transfer that to tie_origin
                                // (turn on TieStart)
                                if cur_note.tie_info.intersects(TieInfo::TieStart) {
                                    potential_tie_origin.tie_info |= TieInfo::TieStart;
                                }

                                // TODO: join lyrics as well ??
                            } else {
                                // In case potential note to be joined is not joinable,
                                // it is probably a slur and so we simply append note to stream
                                note_stream.push(cur_note);
                            }
                        } else {
                            // If potential note to be joined does not want tobe joined, simply append
                            note_stream.push(cur_note);
                        }
                    } else {
                        // If TieEnd is there but no note to join with
                        // -> simply append note to stream
                        note_stream.push(cur_note);
                    }
                } else if cur_note.tie_info.intersects(TieInfo::TieStart)
                {
                    note_stream.push(cur_note);
                }
                else {unreachable!()}
            }
        }
        Ok(note_stream)
    }
    pub fn fuse_tied_notes(&self) -> anyhow::Result<Self>
    {
        let mut part = Part::new(
            self.name.clone(),
            self.key_sig,
            self.clef_sign,
            self.time_sig
        );
        part.gnotes.reserve(self.gnotes.len());

        let mut _gnotes = RefCell::new(VecDeque::from(self.gnotes.clone()));
        while !_gnotes.borrow().is_empty() {
            match _gnotes.borrow_mut().pop_front().unwrap() {
                Gnote::SimpleNote(sn) => {
                    let mut tmp_snote_stream
                        = SmallVec::<[SimpleNote; 20]>::new();
                    tmp_snote_stream.push(sn);
                    while let Some(Gnote::SimpleNote(new_sn)) = _gnotes.borrow().front() {
                        tmp_snote_stream.push(
                            match _gnotes.borrow_mut().pop_front().unwrap() {
                                Gnote::SimpleNote(sn) => {sn},
                                _ => {panic!()}
                            }
                        );
                    }

                    let note_stream
                        = Self::fuse_tied_notes_in_range(tmp_snote_stream.as_slice())?;
                    part
                    .gnotes
                    .extend(
                        note_stream
                        .into_iter()
                        .map(|sn| {Gnote::from(sn)})
                    )
                }
                Gnote::Tuplet(mut tup) => {
                    let note_stream
                        = Self::fuse_tied_notes_in_range(tup.notes.as_slice())?;

                    tup.notes.clear();
                    tup
                    .notes
                    .extend(note_stream.into_iter())
                }
            }
        }
        Ok(part)
    }

    pub fn transpose_by(&self, displacement: PsType) -> Self
    {
        let mut part = self.clone();
        part
        .simple_note_mut_iter()
        .for_each(|sn| {
            sn.pitches
             = BTreeSet::from_iter(
                sn.pitches
                .iter()
                .map(|pt| {
                    let mut _pt = pt.clone();
                    _pt.transpose(displacement);
                    _pt
                })
            )
        });
        part
    }

    pub fn simple_note_mut_iter(&mut self)
        -> impl Iterator<Item=&mut simple_note::SimpleNote>
    {
        let mut residue = VecDeque::<&mut simple_note::SimpleNote>::new();
        residue.reserve(config::EXP_TUP_LEN);
        let mut cur_gnote = self.gnotes.iter_mut();

        from_fn(move || {
            return
            if residue.is_empty() {
                let new_note = cur_gnote.next();
                if new_note.is_some() {
                    match new_note.unwrap() {
                        Gnote::SimpleNote(ref mut sn) => Some(sn),
                        Gnote::Tuplet(ref mut tup) => {
                            tup
                            .notes
                            .iter_mut()
                            .for_each(|sn| {residue.push_back(sn)});

                            return residue.pop_front();
                        }
                    }
                } else { None }
            } else { residue.pop_front() }
        })
    }

    // forgive me heavenly father for I have sinned
    pub fn simple_note_iter(&self)
        -> impl Iterator<Item=&simple_note::SimpleNote>
    {
        // should be fine since we did not do any modification and the user of this can't either
        unsafe {
            let mut mut_self = &mut *( (self as *const Self) as *mut Self);
            mut_self
            .simple_note_mut_iter()
            .map(|sn| { &*(sn as *const simple_note::SimpleNote)})
        }
    }

    pub fn hash_iter<'a, D: Digest + Default, SnIter: Iterator<Item=&'a SimpleNote>>
        (iter: SnIter) -> SmallVec<[u8; 128]>
    {
        let mut sha = D::default();
        unsafe {
            for sn in iter {
                macro_rules! hash_type {
                    ($hasher:ident, $value:expr, $value_type:ty) => {{
                        $hasher.update(
                            std::mem::transmute::<$value_type, [u8; size_of::<$value_type>()/size_of::<u8>()]>
                            ($value)
                        );
                    }};
                }

                // Hash pitch
                if sn.is_rest() {
                    sha.update("R");
                } else {
                    for pitch in sn.pitches.iter() {
                        hash_type!(sha, pitch.ps, PsType);
                    }
                }
                
                // Hash duration
                hash_type!(sha, *sn.interval.start.numer(), BeatDivision);
                hash_type!(sha, *sn.interval.start.denom(), BeatDivision);
                hash_type!(sha, *sn.interval.end.numer(), BeatDivision);
                hash_type!(sha, *sn.interval.end.denom(), BeatDivision);

                // TODO: hash lyrics
            }
        }
        SmallVec::from(sha.finalize().as_slice())
    }

    fn hash_iter_512<'a, SnIter: Iterator<Item=&'a SimpleNote>>
        (iter: SnIter)
        -> SmallVec<[u8; 128]>
    {
        Self::hash_iter::<Sha512, SnIter>(iter)
    }

    pub fn hash_note_and_rests(&self) -> SmallVec<[u8; 128]> {
        Self::hash_iter_512(self.simple_note_iter())
    }
}

pub struct MeasuredPart {
    pub name: String,
    pub key_sig: KeySignature,
    pub clef_sign: ClefType,
    pub time_sig: TimeSig,
    pub measures: Vec<Measure>
}

impl MeasuredPart {
    pub fn new(
        part_name: String,
        key_sig: KeySignature,
        clef_sign: ClefType,
        time_sig: TimeSig
    ) -> Self
    {
        Self {
            name: part_name,
            key_sig,
            clef_sign,
            time_sig,
            measures: Vec::new()
        }
    }

    pub fn flatten(&self) -> Part {
        let mut flat_part = Part::new(
            self.name.clone(),
            self.key_sig,
            self.clef_sign,
            self.time_sig
        );

        let mut acc_offset = Offset::from_integer(0);
        for measure in self.measures.iter() {
            for gn in measure.gnotes.iter() {
                let mut new_gnote = gn.clone();
                match &mut new_gnote {
                    Gnote::SimpleNote(sn) => sn.interval.set_start_keep_length(acc_offset),
                    Gnote::Tuplet(tup) => tup.interval.set_start_keep_length(acc_offset)
                }
                acc_offset += either_gnote!(&new_gnote, gn => gn.interval.length);
                flat_part.gnotes.push(new_gnote);
            }
        }
        flat_part
    }

    pub fn append_gnote(&mut self, gnote: Gnote) -> anyhow::Result<()>
    {
        let measure_length
            = Duration::new((self.time_sig.numer() * 4) as BeatDivision,
                            *self.time_sig.denom() as BeatDivision);

        if self.measures.is_empty() {
            self.measures.push(
                Measure::new(
                Offset::from_integer(0),
                measure_length,
                0 as MeasureNumberType,
                Vec::new()
                )
            );
        }

        let mut _buffer = RefCell::new(VecDeque::with_capacity(5));
        _buffer.borrow_mut().push_back(gnote);
        while let Some(mut cur_gnote) = _buffer.borrow_mut().pop_front()
        {
            let current_measure_window = self.measures.last().unwrap().interval;

            {
                let current_measure = self.measures.last().unwrap();
                either_gnote!(&mut cur_gnote, gn => gn.interval)
                .set_start_keep_length(
                    if current_measure.gnotes.is_empty()
                    { current_measure_window.start }
                    else
                    {
                        current_measure_window.start
                        + either_gnote!(current_measure.gnotes.last().unwrap(), gn => gn.interval.end)
                    }
                );
            }

            if current_measure_window
                .does_swallow(either_gnote!(&cur_gnote, gn => gn.interval).borrow())
            {
                let mut shifted_gnote = cur_gnote;
                either_gnote!(&mut shifted_gnote, gn => gn.interval)
                .displace_start_keep_length(-current_measure_window.start);

                self
                .measures
                .last_mut()
                .unwrap()
                .gnotes
                .push(shifted_gnote);
            }
            else if current_measure_window.end == either_gnote!(&cur_gnote, gn => gn.interval).start
            {
                {
                    let current_measure = self.measures.last().unwrap();
                    self.measures.push(
                        Measure::new(
                            current_measure_window.end,
                            measure_length,
                            current_measure.measure_number + 1,
                            Vec::new()
                        )
                    );
                }

                let new_current_measure = self.measures.last_mut().unwrap();

                either_gnote!(&mut cur_gnote, gn => gn.interval)
                    .displace_start_keep_length(-new_current_measure.interval.start);

                new_current_measure.gnotes.push(cur_gnote);
            }
            else if current_measure_window
                .does_overlap_with(either_gnote!(&cur_gnote, gn => gn.interval).borrow())
            {
                let (mut first_half, mut second_half)
                    = either_gnote!(cur_gnote, gn => gn.split_at_offset(current_measure_window.end));

                // Shift offset
                [&mut first_half, &mut second_half]
                    .iter_mut()
                    .flat_map(|half| {half.iter_mut()})
                    .for_each(
                        |snote| {
                            snote.interval.displace_start_keep_length(-current_measure_window.start)
                        }
                    );

                self
                    .measures
                    .last_mut()
                    .unwrap()
                    .gnotes
                    .extend(
                        first_half
                        .into_iter()
                        .map( |sn| { Gnote::from(sn) } )
                    );

                let current_measure_number
                = self
                    .measures
                    .last()
                    .unwrap()
                    .measure_number;

                self
                    .measures
                    .push(
                        Measure::new(
                            current_measure_window.end,
                            measure_length,
                            current_measure_number + 1,
                            Vec::new()
                        )
                    );

                // surgery to retains invariant
                {
                    let mut mut_gnotes = _buffer.borrow_mut();
                    mut_gnotes.reserve(second_half.len());
                    second_half
                        .into_iter()
                        .rev()
                        .map(|sn| { Gnote::SimpleNote(sn) })
                        .for_each(|gn| { mut_gnotes.push_front(gn) })
                }
            }
        }
        assert!(
            self
            .measures
            .iter()
            .all( |mea| {mea.get_elements_acc_duration() == measure_length} )
        );
        Ok(())
    }
}