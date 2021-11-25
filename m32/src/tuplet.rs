use fraction::Ratio;
use smallvec::SmallVec;
use crate::attribs::{BeatDivision, Duration, MPInterval, Offset};
use crate::simple_note::{SimpleNote};
use crate::config::*;

pub type NormalNumType = u16;

#[derive(Clone)]
pub struct Tuplet {
    pub notes: Vec<SimpleNote>,
    pub interval: MPInterval, // should not be changed directly
    pub actual_number: NormalNumType,
    pub normal_number: NormalNumType
}

impl Tuplet {
    pub fn new(
        normal_number: NormalNumType,
        actual_number: NormalNumType,
        interval: MPInterval,
        simple_notes: Vec<SimpleNote>
    ) -> Self {
        Self {
            notes: simple_notes,
            interval,
            actual_number,
            normal_number
        }
    }

    pub fn displace_start_keep_length(&mut self, displacement: Duration) {
        self.displace_start_keep_length(displacement);
        self.notes.iter_mut()
        .for_each(|nt|
            {nt.interval.displace_start_keep_length(displacement)}
        )
    }

    pub fn set_start_keep_length(&mut self, start: Offset) {
        let displacement = start - self.interval.start;
        self.set_start_keep_length(start);
        self.notes.iter_mut()
        .for_each(|nt| {nt.interval.displace_start_keep_length(displacement)})
    }

    pub fn flatten(&self) -> SmallVec<[SimpleNote; config::EXP_TUP_LEN]>
    {
        // truncate right
        let normal_note_duration =
            self.interval.length / Offset::from_integer(self.normal_number as BeatDivision);
        let mut flat = SmallVec::<[SimpleNote; config::EXP_TUP_LEN]>::new();
        flat.reserve(self.notes.len());

        let mut last_offset = self.interval.start;
        for note in self.notes.iter() {
            let mut new_note = note.clone();
            new_note.interval.set_start_keep_length(last_offset);
            new_note.interval.set_length_keep_start(normal_note_duration);
            last_offset += normal_note_duration;

            flat.push(new_note);
        }
        flat
    }

    pub fn split_at_offset(&self, offset: Offset)
        -> (SmallVec<[SimpleNote; config::EXP_TUP_LEN]>,
            SmallVec<[SimpleNote; config::EXP_TUP_LEN]>)
    {
        assert_eq!(
            self.notes
                .iter()
                .fold(Duration::from(0),
                      |length, note| {
                            length + note.interval.length
                      }),
            self.interval.length,
            "tuplet components don't add up length-wise!"
        );
        let mut flat_vec = self.flatten();

        for (index, note) in
            flat_vec
            .iter()
            .enumerate()
            .filter(
                |(index, note)|
                {note.interval.does_half_closed_contains_offset(offset)})
        {
            // if already cut
            if note.interval.start == offset {
                let (first_half, second_half) = flat_vec.split_at(index);
                return (first_half.into(), second_half.into());
            }
            let crumbles = note.split_at_offset(offset);
            let (first_half, second_half) = flat_vec.split_at(index);
            let (mut first, mut second)
                = (SmallVec::<[SimpleNote; 6]>::from(first_half),
                   SmallVec::<[SimpleNote; 6]>::from(second_half));
            first.pop();
            first.extend(crumbles.0.into_iter());
            crumbles.1
            .into_iter()
            .rev()
            .for_each(|sn| {second.insert(0, sn)});
            return (first, second);
        }
        unreachable!()
    }
}