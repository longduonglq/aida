pub type PitchClass = i8;
pub type Octave = i8;
pub type PsType = i16;

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

pub enum Alter {
    Flat = -1,
    No = 0,
    Sharp = 1
}

pub struct Pitch {
    step: DiatonicStep,
    octave: Option<Octave>,
    alter: Alter,
    ps: PsType
}

impl Pitch {
    pub fn transpose(&mut self, half_steps: PsType) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn update_ps(&mut self, ps: PsType) {}
}
