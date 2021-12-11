use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct Color {
    red: u16,
    green: u16,
    blue: u16
}

impl Color {
    pub fn from_hex_rgb(hex_str: &str) -> Self {
        let mut processed = hex_str.strip_prefix("#").unwrap();
        // strip alpha if present
        if processed.len() == 8 { processed = &processed[2..] }
        assert_eq!(processed.len(), 6);
        Color {
            red: u16::from_str_radix(&processed[0..2], 16).unwrap(),
            green: u16::from_str_radix(&processed[2..4], 16).unwrap(),
            blue: u16::from_str_radix(&processed[4..6], 16).unwrap(),
        }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:X}{:X}{:X}", self.red, self.green, self.blue)
    }
}

impl From<&str> for Color {
    fn from(hex: &str) -> Self {
        Color::from_hex_rgb(hex)
    }
}

impl Into<String> for Color {
    fn into(self) -> String {
        self.to_hex()
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    }
}

#[cfg(test)]
mod tests {
    use crate::color::Color;

    #[test]
    fn test1() {
        let color1 = Color::from_hex_rgb("#32a852");
        assert!(
            color1.red == 50&&
            color1.green == 168&&
            color1.blue == 82
        );

        let hex1 = Color::to_hex(&color1);
        assert_eq!(hex1.to_lowercase(), "#32a852".to_lowercase())
    }

    #[test]
    fn test2() {
        let color1 = Color::from_hex_rgb("#3532A852");
        assert!(
            color1.red == 50&&
            color1.green == 168&&
            color1.blue == 82
        );

        let hex1 = Color::to_hex(&color1);
        assert_eq!(hex1.to_lowercase(), "#32a852".to_lowercase())
    }
}
