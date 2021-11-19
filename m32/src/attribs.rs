use fraction::{GenericFraction, Ratio};
use crate::interval::PInterval;
use super::interval;

pub type KeySignature = i8;
pub type BeatDivision = i32;

pub type TimeSigComponent = u8;
pub type TimeSig = Ratio<TimeSigComponent>;

pub type ClefType = i8;
pub type Offset = Ratio<BeatDivision>;
pub type Duration = Offset;
pub type MPInterval = PInterval<Offset>;