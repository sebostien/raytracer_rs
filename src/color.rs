use serde::{Deserialize, Serialize};

/// RGB color
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Color {
    /// [0, 255]
    r: f64,
    /// [0, 255]
    g: f64,
    /// [0, 255]
    b: f64,
}

pub const WHITE_COLOR: Color = Color {
    r: 255.0,
    g: 255.0,
    b: 255.0,
};

pub const BLACK_COLOR: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
};

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            r: red as f64,
            g: green as f64,
            b: blue as f64,
        }
    }

    pub fn scale(&self, s: f64) -> Self {
        Self {
            r: (self.r * s).min(255.0),
            g: (self.g * s).min(255.0),
            b: (self.b * s).min(255.0),
        }
    }

    pub fn add(mut self, other: &Color) -> Self {
        self.r += other.r;
        self.g += other.g;
        self.b += other.b;
        self
    }
}

impl From<Color> for [u8; 3] {
    fn from(value: Color) -> Self {
        debug_assert!(0.0 <= value.r && value.r <= 255.0);
        debug_assert!(0.0 <= value.g && value.g <= 255.0);
        debug_assert!(0.0 <= value.b && value.b <= 255.0);
        [
            value.r.round() as u8,
            value.g.round() as u8,
            value.b.round() as u8,
        ]
    }
}
