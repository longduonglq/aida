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

impl DurationName {
}