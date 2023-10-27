use std::str::FromStr;

use raytrace_lib::color::ColorNames;
use raytrace_lib::{Color, Vec3};

use crate::options::Options;
use crate::{Ident, SceneParseError};

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedLit {
    pub start: usize,
    lit: Lit,
    pub end: usize,
}

impl SpannedLit {
    pub fn new(start: usize, lit: Lit, end: usize) -> Self {
        Self { start, lit, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    String(String),
    Double(f64),
    Int(i32),
    Tuple(Vec<SpannedLit>),
    Object(Vec<(Ident, SpannedLit)>),
}

const TYPE_STRING: &str = "Str";
const TYPE_DOUBLE: &str = "f64";
const TYPE_VEC3: &str = "( f64, f64, f64 )";
const TYPE_COLOR: &str = "( u8, u8, u8 )";
const TYPE_INT: &str = "int";
const TYPE_U32: &str = "u32";
const TYPE_U8: &str = "u8";
const TYPE_OBJECT: &str = "{}";

impl std::fmt::Display for SpannedLit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.lit {
            Lit::String(s) => write!(f, "{s}"),
            Lit::Double(d) => write!(f, "{d}"),
            Lit::Int(d) => write!(f, "{d}"),
            Lit::Tuple(t) => write!(
                f,
                "( {} )",
                t.iter()
                    .map(|s| format!("{s}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Lit::Object(os) => write!(
                f,
                "{{ {} }}",
                os.iter()
                    .map(|(k, v)| format!("{}: {v}", k.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl SpannedLit {
    fn to_type_string(&self) -> String {
        match &self.lit {
            Lit::String(_) => TYPE_STRING.to_string(),
            Lit::Double(_) => TYPE_DOUBLE.to_string(),
            Lit::Int(_) => TYPE_INT.to_string(),
            Lit::Tuple(v) => format!(
                "( {} )",
                v.iter()
                    .map(|l| l.to_type_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Lit::Object(os) => format!(
                "{{ {} }}",
                os.iter()
                    .map(|(k, v)| format!("{}: {}", k.name, v.to_type_string()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }


    pub fn get_string(&self) -> Result<String, SceneParseError> {
        match &self.lit {
            Lit::String(s) => Ok(s[1..s.len() - 1].to_string()),
            _ => Err(SceneParseError::WrongType {
                start: self.start,
                t: self.to_type_string(),
                expected: TYPE_STRING,
                end: self.end,
            }),
        }
    }

    // TODO: Use a macro for all get_{number}
    pub fn get_double(&self) -> Result<f64, SceneParseError> {
        match self.lit {
            Lit::Double(d) => Ok(d),
            Lit::Int(d) => Ok(d.into()),
            _ => Err(SceneParseError::WrongType {
                start: self.start,
                t: self.to_type_string(),
                expected: TYPE_DOUBLE,
                end: self.end,
            }),
        }
    }

    pub fn get_u32(&self) -> Result<u32, SceneParseError> {
        match self.lit {
            Lit::Int(i) => {
                match u32::try_from(i) {
                    Ok(o) => Ok(o),
                    Err(err) => Err(SceneParseError::Custom {
                        start: self.start,
                        error: format!("{}", err),
                        end: Some(self.end),
                    }),
                }
                // message: format!("Number must be positive: '{i}'"),
            }
            _ => Err(SceneParseError::WrongType {
                start: self.start,
                t: self.to_type_string(),
                expected: TYPE_U32,
                end: self.end,
            }),
        }
    }

    pub fn get_u8(&self) -> Result<u8, SceneParseError> {
        match &self.lit {
            &Lit::Int(i) => match u8::try_from(i) {
                Ok(n) => Ok(n),
                Err(err) => Err(SceneParseError::Custom {
                    start: self.start,
                    error: format!("{}", err),
                    end: Some(self.end),
                }),
            },
            _ => Err(SceneParseError::WrongType {
                start: self.start,
                t: self.to_type_string(),
                expected: TYPE_U8,
                end: self.end,
            }),
        }
    }

    pub fn get_vec3(&self) -> Result<Vec3, SceneParseError> {
        if let Lit::Tuple(v) = &self.lit {
            if let [x, y, z] = v.as_slice() {
                return Ok(Vec3::new(x.get_double()?, y.get_double()?, z.get_double()?));
            }
        }

        Err(SceneParseError::WrongType {
            start: self.start,
            t: self.to_type_string(),
            expected: TYPE_VEC3,
            end: self.end,
        })
    }

    pub fn get_color(&self) -> Result<Color, SceneParseError> {
        match &self.lit {
            // Either "red"
            Lit::String(name) => {
                let color =
                    ColorNames::from_str(name).map_err(|_| SceneParseError::UnknownColor {
                        start: self.start,
                        name: name.clone(),
                        end: self.end,
                    })?;
                return Ok(color.into());
            }
            // Or tuple (255,0,0)
            Lit::Tuple(color) => {
                if let [x, y, z] = color.as_slice() {
                    return Ok(Color::new(x.get_u8()?, y.get_u8()?, z.get_u8()?));
                }
            }
            _ => {}
        }

        Err(SceneParseError::WrongType {
            start: self.start,
            t: self.to_type_string(),
            expected: TYPE_COLOR,
            end: self.end,
        })
    }
}

impl TryFrom<SpannedLit> for Options {
    type Error = SceneParseError;

    fn try_from(value: SpannedLit) -> Result<Self, Self::Error> {
        if let Lit::Object(os) = value.lit {
            Ok(Self::build(os)?)
        } else {
            Err(SceneParseError::WrongType {
                start: value.start,
                t: value.to_type_string(),
                expected: TYPE_OBJECT,
                end: value.end,
            })
        }
    }
}
