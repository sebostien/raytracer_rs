use std::str::FromStr;

use crate::Color;

#[derive(Debug)]
pub struct Material {
    pub color: Color,
    /// Specular reflection defines how much of light the object reflects.
    /// Should be in range \[0,1\].
    /// <https://en.wikipedia.org/wiki/Specular_reflection>
    pub specular: f64,
    /// Lamberterian reflectance defines how “matte” the object appears.
    /// Should be in range \[0,1\].
    /// <https://en.wikipedia.org/wiki/Lambertian_reflectance>
    pub lambert: f64,
    /// Ambient lighting defines how strong the “base light” should be interpreted.
    /// Should be in range \[0,1\].
    /// <https://en.wikipedia.org/wiki/Shading#Ambient_lighting>
    pub ambient: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialTemplate {
    Matte,
    Metal,
    Mirror,
}

impl FromStr for MaterialTemplate {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use MaterialTemplate::*;
        let m = match s {
            "matte" => Matte,
            "metal" => Metal,
            "mirror" => Mirror,
            _ => return Err(format!("No material template named '{s}'")),
        };
        Ok(m)
    }
}

impl MaterialTemplate {
    pub fn get_name_tuples() -> [(&'static str, Self); 3] {
        use MaterialTemplate::*;

        [("matte", Matte), ("metal", Metal), ("mirror", Mirror)]
    }

    pub fn get_material(&self, color: Color) -> Material {
        use MaterialTemplate::*;

        match self {
            Matte => Material {
                color,
                specular: 0.0,
                lambert: 0.9,
                ambient: 0.2,
            },
            Metal => Material {
                color,
                specular: 0.8,
                lambert: 0.3,
                ambient: 0.1,
            },
            Mirror => Material {
                color,
                specular: 1.0,
                lambert: 0.0,
                ambient: 0.0,
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
