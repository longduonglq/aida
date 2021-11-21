#[derive(Clone)]
pub struct Lyric {
    pub number: u8,
    pub text: String
}

impl Lyric {
    pub fn new(number: u8, text: String) -> Self {
        Self { number, text }
    }
}