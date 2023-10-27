use std::str::FromStr;

use crate::Color;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: Color,
    /// Specular reflection defines how much of light the object reflects.
    /// <https://en.wikipedia.org/wiki/Specular_reflection>
    pub specular: Color,
    /// Lamberterian reflectance defines how “matte” the object appears.
    /// <https://en.wikipedia.org/wiki/Lambertian_reflectance>
    pub lambert: Color,
    /// Ambient lighting defines how strong the “base light” should be interpreted.
    /// <https://en.wikipedia.org/wiki/Shading#Ambient_lighting>
    pub ambient: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialTemplate {
    Red,
    Green,
    Blue,
    Bronze,
}

impl FromStr for MaterialTemplate {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use MaterialTemplate::{Red, Green, Blue, Bronze};
        let m = match s {
            "red" => Red,
            "green" => Green,
            "blue" => Blue,
            "bronze" => Bronze,
            _ => return Err(format!("No material template named '{s}'")),
        };
        Ok(m)
    }
}

impl MaterialTemplate {
    pub fn get_name_tuples() -> [(&'static str, Self); 4] {
        use MaterialTemplate::{Red, Green, Blue, Bronze};

        [
            ("red", Red),
            ("green", Green),
            ("blue", Blue),
            ("bronze", Bronze),
        ]
    }

    pub fn get_material(&self, color: Color) -> Material {
        use MaterialTemplate::{Red, Green, Blue, Bronze};

        match self {
            Red => Material {
                color,
                ambient: Color::zero(),
                lambert: Color::new_f(1.0, 0.0, 0.0),
                specular: Color::new_f(0.0225, 0.0225, 0.0225),
            },
            Green => Material {
                color,
                ambient: Color::zero(),
                lambert: Color::new_f(0.0, 1.0, 0.0),
                specular: Color::new_f(0.0225, 0.0225, 0.0225),
            },
            Blue => Material {
                color,
                ambient: Color::zero(),
                lambert: Color::new_f(0.0, 0.0, 1.0),
                specular: Color::new_f(0.0225, 0.0225, 0.0225),
            },
            Bronze => Material {
                color,
                ambient: Color::new_f(0.2125, 0.1275, 0.054),
                lambert: Color::new_f(0.714, 0.4284, 0.18144),
                specular: Color::new_f(0.393548, 0.271906, 0.166721),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MaterialTemplate;

    #[test]
    fn all_materials_have_names() {
        for (s, m) in MaterialTemplate::get_name_tuples() {
            assert_eq!(m, s.parse().unwrap());
        }
    }
}
