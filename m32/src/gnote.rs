use crate::simple_note::SimpleNote;
use super::{simple_note, tuplet};

#[macro_export]
macro_rules! either_gnote {
    ($value:expr, $identifier:ident => $result:expr) => {(
        match $value {
            Gnote::SimpleNote($identifier) => $result,
            Gnote::Tuplet($identifier) => $result,
        }
    )}
}

#[derive(Clone)]
pub enum Gnote {
    SimpleNote(simple_note::SimpleNote),
    Tuplet(tuplet::Tuplet)
}
