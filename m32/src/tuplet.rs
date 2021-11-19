use fraction::Ratio;
use smallvec::SmallVec;
use crate::attribs::{Duration, MPInterval, Offset};
use crate::simple_note::{SimpleNote};
use crate::config::*;

type NormalNumType = u16;

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

    // fn set_start_keep_end(&mut self, _start: Offset) {
    //     let displacement = _start - self._interval.start;
    //     self._interval.set_start_keep_end(_start);
    //     for note in self.notes.iter_mut() {
    //         note.interval.displace_start_keep_end(displacement);
    //     }
    // }

    pub fn flatten(&self) -> SmallVec<[SimpleNote; config::EXP_TUP_LEN]>
    {
        // truncate right
        let normal_note_duration =
            self.interval.length / Offset::new(self.normal_number as i32, 1);
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
            let (_first_half, _second_half) = flat_vec.split_at(index);
            let mut first: SmallVec<[SimpleNote; config::EXP_TUP_LEN]> = _first_half.into();
            first.pop();
            first.push(crumbles.0);
            let mut second: SmallVec<[SimpleNote; config::EXP_TUP_LEN]> = _second_half.into();
            second.insert(0, crumbles.1);
            return (first, second);
        }
        unreachable!()
    }
}