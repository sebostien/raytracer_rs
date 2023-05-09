/// RGB color
#[derive(Debug, Clone, Copy)]
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
}

impl std::ops::Add for Color {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl From<Color> for [u8; 3] {
    fn from(value: Color) -> Self {
        #[cfg(debug_assertions)]
        {
            if !(0.0..=255.0).contains(&value.r)
                || !(0.0..=255.0).contains(&value.g)
                || !(0.0..=255.0).contains(&value.b)
            {
                // TODO: Blend better! This happens too often.
                // println!("Color is outside rgb range");
            }
        }
        [
            value.r.round() as u8,
            value.g.round() as u8,
            value.b.round() as u8,
        ]
    }
}
