use fraction::Ratio;
use crate::attribs::{Duration, MPInterval, Offset};
use crate::either_gnote;
use crate::gnote::Gnote;
use crate::simple_note::SimpleNote;
use crate::tuplet::Tuplet;

pub type MeasureNumberType = u32;
#[derive(Clone)]
pub struct Measure {
    pub interval: MPInterval,
    pub gnotes: Vec<Gnote>,
    pub measure_number: MeasureNumberType,
}

impl Measure {
    pub fn new(
        offset: Offset,
        duration: Duration,
        measure_number: MeasureNumberType,
        gnotes: Vec<Gnote>
    ) -> Measure
    {
        Measure {
            interval: MPInterval::from_start_and_length(offset, duration),
            gnotes,
            measure_number
        }
    }

    pub fn get_elements_acc_duration(&self) -> Duration
    {
        self
        .gnotes
        .iter()
        .fold(
            Duration::from_integer(0),
            |acc, gnote|{
                acc + either_gnote!(&gnote, gn => gn.interval.length)
            }
        )
    }

    pub fn is_rest_only(&self) -> bool
    {
        self
        .gnotes
        .iter()
        .all(
            |gnote| {
                match gnote {
                    Gnote::SimpleNote(sn) => sn.is_rest(),
                    Gnote::Tuplet(tup) => {
                        tup
                        .notes
                        .iter()
                        .all(|sn|{sn.is_rest()})
                    }
                }
            }
        )
    }
}

