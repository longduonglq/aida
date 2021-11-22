use crate::part::MeasuredPart;
use super::part::{Part};

pub struct Score {
    title: String,
    parts: Vec<Part>
}

impl Score {
    pub fn new(title: String) -> Self
    {
        Self {
            title,
            parts: Vec::new()
        }
    }

    pub fn to_measured(&self) -> MeasuredScore
    {
        let mut new_score = MeasuredScore::new(self.title.clone());
        new_score.measured_parts.reserve(self.parts.len());
        new_score
        .measured_parts
        .extend(
            self
            .parts
            .iter()
            .map(
                |part| {part.to_measured()}
            )
        );

        new_score
    }

    pub fn fuse_tied_notes(&self) -> anyhow::Result<Self> {
        let mut new_score = Self::new(self.title.clone());
        new_score.parts.reserve(self.parts.len());
        new_score
        .parts
        .extend(
            self
            .parts
            .iter()
            .map(|part| {part.fuse_tied_notes().unwrap()}) // TODO: fix this: ineffective Error handling
        );
        Ok(new_score)
    }
}

pub struct MeasuredScore {
    pub title: String,
    pub measured_parts: Vec<MeasuredPart>
}

impl MeasuredScore {
    pub fn new(title: String) -> Self {
        Self {
            title,
            measured_parts: Vec::new()
        }
    }

    pub fn flatten(&self) -> Score {
        let mut flat_score = Score::new(self.title.clone());
        flat_score.parts.reserve(self.measured_parts.len());
        flat_score
        .parts
        .extend(
            self
            .measured_parts
            .iter()
            .map(|mpart| {mpart.flatten()})
        );
        flat_score
    }

    // TODO: muust be TESTED
    // these are indexes
    pub fn vertical_crop(&self, start: usize, stop: usize) -> MeasuredScore {
        let mut crop = MeasuredScore::new(self.title.clone());
        crop.measured_parts.reserve(self.measured_parts.len());

        self
        .measured_parts
        .iter()
        .for_each(
            |orig_part| {
                let mut part_clone = MeasuredPart::new(
                    orig_part.name.clone(),
                    orig_part.key_sig,
                    orig_part.clef_sign,
                    orig_part.time_sig
                );
                part_clone.measures.reserve(stop - start);
                part_clone
                .measures
                .extend_from_slice(&orig_part.measures[start..stop])
            }
        );
        crop
    }
}