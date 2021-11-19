use crate::part::MeasuredPart;
use super::part::{Part};

pub struct Score {
    title: String,
    parts: Vec<Part>
}

pub struct MeasuredScore {
    title: String,
    measured_parts: Vec<MeasuredPart>
}