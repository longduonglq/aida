use fraction::GenericFraction;
use crate::interval::PInterval;
use super::interval;

type KeySignature = i8;
type BeatDivision = i32;

type TimeSigComponent = u8;
type TimeSig = GenericFraction<TimeSigComponent>;

pub type ClefType = i8;
pub type Offset = GenericFraction<BeatDivision>;
pub type Duration = Offset;
pub type MPInterval = PInterval<Offset>;