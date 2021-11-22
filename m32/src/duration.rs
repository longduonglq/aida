use std::collections::BTreeMap;
use smallvec::{SmallVec, smallvec};
use super::attribs::Duration;
use fraction::*;

#[derive(Copy, Clone)]
enum DurationName {
    Unspecified,
    Maxima,
    Longa,
    Breve,
    Whole,
    Half,
    Quarter,
    Eighth,
    Dur16th,
    Dur32nd,
    Dur64th,
    Dur128th,
    Dur256th,
    Dur512nd,
    Dur1024th
}

impl Into<&str> for DurationName {
    fn into(self) -> &'static str {
        match self {
            DurationName::Maxima => "maxima",
            DurationName::Longa => "longa",
            DurationName::Breve => "breve",
            DurationName::Whole => "whole",
            DurationName::Half => "half",
            DurationName::Quarter => "quarter",
            DurationName::Eighth => "eight",
            DurationName::Dur16th => "16th",
            DurationName::Dur32nd => "32nd",
            DurationName::Dur64th => "64th",
            DurationName::Dur128th => "128th",
            DurationName::Dur256th => "256th",
            DurationName::Dur512nd => "512nd",
            DurationName::Dur1024th => "1024th",
            _ => panic!("Unknown duration")
        }
    }
}

lazy_static! {
    static ref DURATION_TO_DURATION_NAME: BTreeMap<Duration, DurationName> = {
        let mut m = BTreeMap::new();
        m.insert(Duration::new(32, 1), DurationName::Maxima);
        m.insert(Duration::new(16, 1), DurationName::Longa);
        m.insert(Duration::new(8, 1), DurationName::Breve);
        m.insert(Duration::new(4, 1), DurationName::Whole);
        m.insert(Duration::new(2, 1), DurationName::Half);
        m.insert(Duration::new(1, 1), DurationName::Quarter);
        m.insert(Duration::new(1, 2), DurationName::Eighth);
        m.insert(Duration::new(1, 4), DurationName::Dur16th);
        m.insert(Duration::new(1, 8), DurationName::Dur32nd);
        m.insert(Duration::new(1, 16), DurationName::Dur64th);
        m.insert(Duration::new(1, 32), DurationName::Dur128th);
        m.insert(Duration::new(1, 64), DurationName::Dur256th);
        m.insert(Duration::new(1, 128), DurationName::Dur512nd);
        m.insert(Duration::new(1, 256), DurationName::Dur1024th);

        m
    };
}

impl From<Duration> for DurationName {
    fn from(duration: Duration) -> Self {
        DURATION_TO_DURATION_NAME
            .get(&duration)
            .expect("Unknown duration")
            .clone()
    }
}

mod duration_utils {
    use smallvec::SmallVec;
    use crate::attribs::{BeatDivision, Duration};
    use std::ops::Bound::*;
    use fraction::Ratio;
    use crate::duration::{DURATION_TO_DURATION_NAME, DurationName};

    pub fn maximal_extractable_primitive_duration(duration: &Duration) -> Duration {
        DURATION_TO_DURATION_NAME
            .range((Unbounded, Included(duration)))
            .last()
            .expect("Cannot find maximal extractable primitive duration")
            .0
            .clone()
    }

    // Decompose a duration so that it is representable in conventional duration type
    pub fn decompose_duration_into_primitives(duration: &Duration)
        -> Result<SmallVec<[(Duration, u8); 4]>, anyhow::Error>
    {
        let mut decomposition = SmallVec::<[(Duration, u8); 4]>::new();

        // basically division
        let mut remainder = duration.clone();
        let mut max_extractable_primitive = maximal_extractable_primitive_duration(&remainder);
        while remainder >= max_extractable_primitive {
            remainder -= max_extractable_primitive;
            if !decomposition.is_empty() {
                let mut last_component
                    = decomposition
                    .last_mut()
                    .expect("check was done in if above");
                if max_extractable_primitive ==
                    last_component.0 / ((2 as i32).pow((last_component.1 + 1) as i32 as u32))
                {
                    last_component.1 += 1;
                }
                else { decomposition.push((max_extractable_primitive, 0)); }
            }
            else { decomposition.push((max_extractable_primitive, 0)); }

            max_extractable_primitive = maximal_extractable_primitive_duration(&remainder);
        }
        if remainder != Duration::from_integer(0) {
            return Result::Err(anyhow::anyhow!("Duration not representable in primitives"));
        }
        return Ok(decomposition);
    }

    pub fn compute_dotted_length(duration: Duration, mut dots: u8) -> Duration
    {
        assert!(dots >= 0);
        let mut res = Duration::from_integer(0);
        while dots >= 0 {
            res += (duration / (2 as BeatDivision).pow(dots as u32));
            dots -= 1;
        }
        res
    }
}