#[derive(Clone)]
pub struct Color {
    red: u16,
    green: u16,
    blue: u16
}

impl Color {
    fn from_hex_rgb(hex_str: &str) -> Self {
        let mut processed = hex_str.strip_prefix("#").unwrap();
        if processed.len() == 8 {
            processed = &processed[2..];
        }
        Color {
            red: u16::from_str_radix(&processed[1..=2], 16).unwrap(),
            green: u16::from_str_radix(&processed[3..=4], 16).unwrap(),
            blue: u16::from_str_radix(&processed[5..=6], 16).unwrap(),
        }
    }

    fn to_hex(&self) -> String {
        format!("#{:X}{:X}{:X}", self.red, self.green, self.blue)
    }
}