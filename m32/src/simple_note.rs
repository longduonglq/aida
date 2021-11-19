use std::collections::BTreeSet;
use std::fmt::{Debug, Formatter};
use super::attribs::*;
use bitflags::bitflags;
use smallvec::SmallVec;
use crate::pitch::Pitch;
use crate::lyric::Lyric;
use crate::color::*;
use crate::gnote::Gnote;

bitflags! {
    pub struct TieInfo: u8 {
        const TieNeither = 0b00;
        const TieStart = 0b01;
        const TieEnd = 0b10;
        const TieBoth = Self::TieStart.bits | Self::TieEnd.bits;
    }
}

#[derive(Clone)]
pub struct SimpleNote {
    pub interval: MPInterval,
    pub pitches: BTreeSet<Pitch>,

    pub tie_info: TieInfo,
    pub lyrics: Vec<Lyric>,
    pub color: Option<Color>,
    pub dynamic: Option<f32>,
}

impl SimpleNote {
    fn new(
        offset: Offset,
        duration: Duration,
        lyrics: Vec<Lyric>,
        color: Option<Color>,
        tie_info: TieInfo
    ) -> Self
    {
        Self {
            interval: MPInterval::from_start_and_length(offset, duration),
            pitches: BTreeSet::new(),
            tie_info,
            lyrics,
            color,
            dynamic: None
        }
    }

    pub fn split_at_offset(&self, split_offset: Offset)
        -> (SimpleNote, SimpleNote)
    {
        assert!(self.interval.does_half_closed_contains_offset(split_offset));
        assert!(self.interval.start != split_offset && self.interval.end != split_offset);

        let mut left = self.clone();
        let mut right = self.clone();
        left.tie_info |= TieInfo::TieStart;
        right.tie_info |= TieInfo::TieEnd;

        (left, right)
    }

    pub fn is_rest(&self) -> bool { self.pitches.is_empty() }
    pub fn is_note(&self) -> bool { self.pitches.len() == 1 }
    pub fn is_chord(&self) -> bool { self.pitches.len() > 1 }
}

impl Debug for SimpleNote {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_rest() {
            f.write_fmt(format_args!(
                "\tRest<[{:.2}, {:.2}) | length={:.2}>",
                self.interval.start, self.interval.end,
                self.interval.length
            ));
        }
        else if self.is_note() {
            f.write_fmt(format_args!(
                "\tNote<[{:.2}, {:.2}) | length={:.2}> ",
                self.interval.start, self.interval.end,
                self.interval.length
            ));
            format_args!("| {:?}", self.pitches.iter().last().unwrap());
        }
        else {
            f.write_fmt(format_args!(
                "\tChord<[{:.2}, {:.2}) | length={:.2}> ",
                self.interval.start, self.interval.end,
                self.interval.length
            ));
            for pitch in &self.pitches {
                f.write_fmt(format_args!("{:?} ", pitch));
            }
        }
        todo!()
    }
}

impl From<SimpleNote> for Gnote {
    fn from(sn: SimpleNote) -> Self {
        Gnote::SimpleNote(sn)
    }
}

