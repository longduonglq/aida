use fraction::{GenericFraction, Ratio};
use crate::interval::PInterval;
use super::interval;

type KeySignature = i8;
type BeatDivision = i32;

type TimeSigComponent = u8;
type TimeSig = Ratio<TimeSigComponent>;

pub type ClefType = i8;
pub type Offset = Ratio<BeatDivision>;
pub type Duration = Offset;
pub type MPInterval = PInterval<Offset>;