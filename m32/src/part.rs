use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, VecDeque};
use std::iter;
use std::iter::{empty, from_fn, FromIterator};
use anyhow::anyhow;
use smallvec::SmallVec;
use intrusive_collections::LinkedList;
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
    name: String,
    key_sig: KeySignature,
    clef_sign: ClefType,
    time_sig: TimeSig,
    gnotes: Vec<Gnote>
}

impl Part {
    fn new(
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

    fn to_measured(&self) -> MeasuredPart
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
                          self.time_sig.denom().clone() as BeatDivision);
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
        let mut _gnotes = RefCell::new(VecDeque::from(self.gnotes.clone()));
        while let Some(ref cur_gnote) = _gnotes.borrow_mut().pop_front()
        {
            let current_measure_window
                = measured_part
                .measures
                .last_mut()
                .unwrap()
                .interval;

            if current_measure_window
                .does_swallow(either_gnote!(&cur_gnote, gn => gn.interval).borrow())
            {
                let shifted_gnote = cur_gnote.clone();
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

                let shifted_gnote = cur_gnote.clone();
                either_gnote!(&shifted_gnote, gn => gn.interval)
                    .displace_start_keep_length(-new_current_measure.interval.start);

                new_current_measure.gnotes.push(shifted_gnote);
            }
            else if current_measure_window
                .does_overlap_with(either_gnote!(&cur_gnote, gn => &gn.interval))
            {
                let (mut first_half, mut second_half)
                = match cur_gnote {
                    Gnote::Tuplet(tup)
                        => { tup.split_at_offset(current_measure_window.end) },
                    Gnote::SimpleNote(sn)
                        => {
                        let splat = sn.split_at_offset(current_measure_window.end);
                        (SmallVec::from_elem(splat.0, 2),
                         SmallVec::from_elem(splat.1, 2))
                    }
                };

                // Shift offset
                for halves
                    in [&mut first_half, &mut second_half].iter_mut()
                {
                    halves
                    .iter_mut()
                    .for_each(|snote|
                    {
                        snote.interval.displace_start_keep_length(-current_measure_window.start)
                    })
                }

                measured_part
                    .measures
                    .last_mut()
                    .unwrap()
                    .gnotes
                    .extend(
                        first_half
                        .iter()
                        .map( |sn| { Gnote::from(sn.clone()) } )
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
                        .iter()
                        .rev()
                        .map(|sn| { Gnote::SimpleNote(sn.clone()) })
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
        for cur_note in note_range.clone() {
            if cur_note.tie_info == TieInfo::TieNeither {
                note_stream.push(cur_note.clone());
            }
            else {
                if cur_note.tie_info.intersects(TieInfo::TieEnd) {
                    if note_stream.is_empty() {
                        // If TieEnd is there but no note to join with
                        // -> simply append note to stream
                        note_stream.push(cur_note.clone());
                    }
                    else {
                        // If potential note to be joined with exists
                        let mut potential_tie_origin = note_stream.last_mut().unwrap();
                        // If potential note to be joined exhibit TieStart
                        if potential_tie_origin.tie_info.intersects(TieInfo::TieStart) {
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
                            }
                            else {
                                // In case potential note to be joined is not joinable,
                                // it is probably a slur and so we simply append note to stream
                                note_stream.push(cur_note.clone());
                            }
                        }
                        else {
                            // If potential note to be joined does not want tobe joined, simply append
                            note_stream.push(cur_note.clone());
                        }
                    }
                }
                else if cur_note.tie_info.intersects(TieInfo::TieStart)
                {
                    note_stream.push(cur_note.clone());
                }
                else {unreachable!()}
            }
        }
        Ok(note_stream)
    }
    fn fuse_tied_notes(&self) -> anyhow::Result<Self>
    {
        todo!()
    }

    fn transpose_by(&self, displacement: PsType) -> Self
    {
        let mut part = self.clone();
        part
        .simple_note_mut_iter()
        .for_each(|mut sn| {
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

    fn simple_note_mut_iter(&mut self)
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
    fn simple_note_iter(&self)
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
}

pub struct MeasuredPart {
    name: String,
    key_sig: KeySignature,
    clef_sign: ClefType,
    time_sig: TimeSig,
    measures: Vec<Measure>
}

impl MeasuredPart {
    fn new(
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
}