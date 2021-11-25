use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};

pub type PitchClass = i8;
pub type Octave = i8;
pub type PsType = PitchClass;

#[derive(Clone, Copy)]
pub enum DiatonicStep {
    A = 9,
    B = 11,
    C = 0,
    D = 2,
    E = 4,
    F = 5,
    G = 7
}
static DIATONIC_PC: [PitchClass; 7] = [0, 2, 4, 5, 7, 9, 11];

impl From<i8> for DiatonicStep {
    fn from(val: i8) -> Self {
        match val {
            9 => Self::A,
            11 => Self::B,
            0 => Self::C,
            2 => Self::D,
            4 => Self::E,
            5 => Self::F,
            7 => Self::G,
            _ => unreachable!()
        }
    }
}

impl From<&str> for DiatonicStep {
    fn from(s: &str) -> Self {
        match s {
            "A" => Self::A,
            "B" => Self::B,
            "C" => Self::C,
            "D" => Self::D,
            "E" => Self::E,
            "F" => Self::F,
            "G" => Self::G,
            _ => {unreachable!()}
        }
    }
}

impl Into<&str> for DiatonicStep {
    fn into(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
        }
    }
}

#[derive(Clone, Copy)]
pub enum Alter {
    Flat = -1,
    No = 0,
    Sharp = 1
}
impl From<i32> for Alter {
    fn from(num: i32) -> Self {
        match num {
            -1 => Alter::Flat,
            0 => Alter::No,
            1 => Alter::Sharp,
            _ => panic!("Unexpected alter numerical value!")
        }
    }
}

#[derive(Clone)]
pub struct Pitch {
    pub step: DiatonicStep,
    pub octave: Option<Octave>,
    pub alter: Alter,
    pub ps: PsType
}

impl Eq for Pitch {}

impl PartialEq<Self> for Pitch {
    fn eq(&self, other: &Self) -> bool {
        self.ps.eq(&other.ps)
    }
}

impl PartialOrd<Self> for Pitch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ps.partial_cmp(&other.ps)
    }
}

impl Ord for Pitch {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Pitch {
    pub fn new(step: DiatonicStep, octave: Option<Octave>, alter: Alter) -> Self {
        Self {
            step,
            octave,
            alter,
            ps: ((step as i8 + alter as i8) + (octave.unwrap_or(4) + 1) * 12) as PsType
        }
    }
    pub fn transpose(&mut self, half_steps: PsType) {
        self.update_ps(self.ps + half_steps);
    }

    fn update_ps(&mut self, _ps: PsType) {
        let pc: PitchClass = _ps % 12;
        if DIATONIC_PC.binary_search(&pc).is_ok() {
            self.step = DiatonicStep::from(pc);
        }
        else if pc == 1 || pc == 6 || pc == 8 {
            self.alter = Alter::Sharp;
            self.step = DiatonicStep::from(pc - 1);
        }
        else if pc == 10 || pc == 3 {
            self.alter = Alter::Flat;
            self.step = DiatonicStep::from(pc + 1);
        }
        else { unreachable!() }

        if self.octave.is_some() {
            self.octave.replace(((_ps as f32) / 12.0 - 1.0) as i8);
        }
        self.ps = _ps;
    }
}

impl Debug for Pitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(
            format_args!(
                "{:?}{:?}{:?}",
                <DiatonicStep as Into<&str>>::into(self.step),
                {
                    match self.alter {
                        Alter::No => "",
                        Alter::Flat => "b",
                        Alter::Sharp => "#",
                    }
                },
                self.octave.map_or("".to_string(), |r| { r.to_string() })
            )
        )
    }
}
