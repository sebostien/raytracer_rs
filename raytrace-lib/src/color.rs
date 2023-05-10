use std::str::FromStr;

/// RGB color
#[derive(Debug, Clone, Copy)]
pub struct Color {
    /// [0, 1]
    r: f64,
    /// [0, 1]
    g: f64,
    /// [0, 1]
    b: f64,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            r: red as f64 / 255.0,
            g: green as f64 / 255.0,
            b: blue as f64 / 255.0,
        }
    }

    pub fn new_f(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    pub fn zero() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }

    pub fn scale(&self, s: f64) -> Self {
        Self {
            r: (self.r * s).min(1.0),
            g: (self.g * s).min(1.0),
            b: (self.b * s).min(1.0),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.r <= 0.0 && self.g <= 0.0 && self.b <= 0.0
    }
}

impl std::ops::Add for Color {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        Self {
            r: (self.r + rhs.r).min(1.0),
            g: (self.g + rhs.g).min(1.0),
            b: (self.b + rhs.b).min(1.0),
        }
    }
}

impl std::ops::Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl From<Color> for [u8; 3] {
    fn from(value: Color) -> Self {
        debug_assert!(
            (0.0..=1.0).contains(&value.r)
                && (0.0..=1.0).contains(&value.g)
                && (0.0..=1.0).contains(&value.b)
        );
        [
            (value.r * 255.0).round() as u8,
            (value.g * 255.0).round() as u8,
            (value.b * 255.0).round() as u8,
        ]
    }
}

pub enum ColorNames {
    // Base
    White,
    Black,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    // Gold
    Gold,
    GoldenYellow,
    MetallicGold,
    OldGold,
    GoldenPoppy,
}

macro_rules! color {
    ($r:expr,$b:expr,$g:expr) => {
        Color {
            r: $r,
            g: $g,
            b: $b,
        }
    };
}

macro_rules! color_255 {
    ($r:expr,$b:expr,$g:expr) => {
        Color {
            r: ($r / 255u8) as f64,
            g: ($g / 255u8) as f64,
            b: ($b / 255u8) as f64,
        }
    };
}

impl From<ColorNames> for Color {
    fn from(value: ColorNames) -> Self {
        use ColorNames::*;

        match value {
            White => color!(1.0, 1.0, 1.0),
            Black => color!(0.0, 0.0, 0.0),
            Red => color!(1.0, 0.0, 0.0),
            Green => color!(0.0, 1.0, 0.0),
            Blue => color!(0.0, 0.0, 1.0),
            Yellow => color!(1.0, 1.0, 0.0),
            Cyan => color!(0.0, 1.0, 1.0),
            Magenta => color!(1.0, 0.0, 1.0),
            Gold => color_255!(255, 215, 0),
            GoldenYellow => color_255!(255, 223, 0),
            MetallicGold => color_255!(212, 175, 55),
            OldGold => color_255!(207, 181, 59),
            GoldenPoppy => color_255!(252, 194, 0),
        }
    }
}

impl FromStr for ColorNames {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ColorNames::*;

        let color = match s {
            "white" => White,
            "black" => Black,
            "red" => Red,
            "green" => Green,
            "blue" => Blue,
            "yellow" => Yellow,
            "cyan" => Cyan,
            "magenta" => Magenta,
            "gold" => Gold,
            "golden_yellow" => GoldenYellow,
            "metallic_gold" => MetallicGold,
            "old_gold" => OldGold,
            "golden_poppy" => GoldenPoppy,
            _ => {
                return Err(format!("No color named '{}'", s));
            }
        };
        Ok(color)
    }
}
