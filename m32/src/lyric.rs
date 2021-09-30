#[derive(Clone)]
pub struct Lyric {
    number: u8,
    text: String
}

impl Lyric {
    fn new(number: u8, text: String) -> Self {
        Self { number, text }
    }
}