mod lit;
mod options;
mod scene_builder;
mod scene_object;

use lalrpop_util::ParseError;
use raytrace_lib::Raytracer;

#[macro_use]
extern crate lalrpop_util;

// The parser from lalrpop
lalrpop_mod!(
    #[allow(clippy::all)]
    scene
);

const DEFAULT_FOV: f64 = 90.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseStringError {
    UnrecognizedEOF { expected: Vec<String> },
    User { error: String },
    Annotated(String),
    Many(Vec<Self>),
}

impl std::fmt::Display for ParseStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnrecognizedEOF { expected } => {
                write!(
                    f,
                    "error: Unexpected EOF\nExpected one of '{}'",
                    expected.join(", ")
                )
            }
            Self::User { error } => write!(f, "error: {error}"),
            Self::Annotated(error) => {
                write!(f, "{error}")
            }
            Self::Many(errors) => write!(
                f,
                "{}",
                errors.iter().map(|e| format!("{e}\n")).collect::<String>()
            ),
        }
    }
}

impl ParseStringError {
    /// Annotate error like cargo.
    fn annotate(
        source_lines: &[&str],
        start: &Location,
        end: Option<&Location>,
        message: String,
    ) -> Self {
        let line = match source_lines.get(start.line - 1) {
            Some(line) => line,
            None => {
                return Self::Annotated(format!("Line: {}, column: {}", start.line, start.col));
            }
        };

        // Limit output length
        if line.len() > 60 {
            return Self::Annotated(format!("Line: {}, column: {}", start.line, start.col));
        }

        let line_num = start.line.to_string();
        let spaces = " ".repeat(line_num.len());
        let before = " ".repeat(start.col);
        let under = if let Some(end) = end {
            "^".repeat(end.col - start.col)
        } else {
            "".to_string()
        };

        Self::Annotated(format!(
            "
error: {message}
{spaces} |
{} | {line}
{spaces} |{before}{under}
",
            start.line
        ))
    }
}

impl std::error::Error for ParseStringError {}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneParseError {
    UnknownObject {
        start: usize,
        ident: String,
        end: usize,
    },
    UnknownMaterial {
        start: usize,
        name: String,
        end: usize,
    },
    UnknownColor {
        start: usize,
        name: String,
        end: usize,
    },
    DuplicateKey {
        start: usize,
        key: String,
    },
    MissingOption {
        start: usize,
        name: String,
    },
    WrongType {
        start: usize,
        t: String,
        expected: &'static str,
        end: usize,
    },
    UnknownOptions {
        idents: Vec<Ident>,
    },
    Custom {
        start: usize,
        error: String,
        end: Option<usize>,
    },
}

impl SceneParseError {
    pub fn into_parse_string_error(self, input_string: &str) -> ParseStringError {
        let input_lines = &input_string.lines().collect::<Vec<_>>();
        match self {
            SceneParseError::UnknownObject { start, ident, end } => {
                let start = Location::new(start, input_string);
                let end = Location::new(end, input_string);
                ParseStringError::annotate(
                    input_lines,
                    &start,
                    Some(&end),
                    format!("Unknown object '{ident}'"),
                )
            }
            SceneParseError::UnknownMaterial { start, name, end } => {
                let start = Location::new(start, input_string);
                let end = Location::new(end, input_string);
                ParseStringError::annotate(
                    input_lines,
                    &start,
                    Some(&end),
                    format!("Unknown material '{name}'"),
                )
            }
            SceneParseError::UnknownColor { start, name, end } => {
                let start = Location::new(start, input_string);
                let end = Location::new(end, input_string);
                ParseStringError::annotate(
                    input_lines,
                    &start,
                    Some(&end),
                    format!("Unknown color '{name}'"),
                )
            }
            SceneParseError::DuplicateKey { start, key } => {
                let start = Location::new(start, input_string);
                ParseStringError::annotate(
                    input_lines,
                    &start,
                    None,
                    format!("Duplicate key '{key}' in object"),
                )
            }
            SceneParseError::MissingOption { start, name } => {
                let start = Location::new(start, input_string);
                ParseStringError::annotate(
                    input_lines,
                    &start,
                    None,
                    format!("Missing option '{name}' in object"),
                )
            }
            SceneParseError::WrongType {
                start,
                t,
                expected,
                end,
            } => {
                let start = Location::new(start, input_string);
                let end = Location::new(end, input_string);
                ParseStringError::annotate(
                    input_lines,
                    &start,
                    Some(&end),
                    format!("Expected type '{expected}' but found type '{t}'"),
                )
            }

            SceneParseError::UnknownOptions { idents } => ParseStringError::Many(
                idents
                    .into_iter()
                    .map(|Ident { start, name, end }| {
                        let start = &Location::new(start, input_string);
                        let end = Some(Location::new(end, input_string));

                        ParseStringError::annotate(
                            input_lines,
                            start,
                            end.as_ref(),
                            format!("Unkown option '{name}'"),
                        )
                    })
                    .collect(),
            ),
            SceneParseError::Custom { start, error, end } => {
                let start = Location::new(start, input_string);
                let end = end.map(|end| Location::new(end, input_string));

                ParseStringError::annotate(input_lines, &start, end.as_ref(), error)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    start: usize,
    name: String,
    end: usize,
}

impl Ident {
    pub fn new<S: AsRef<str>>(start: usize, name: S, end: usize) -> Self {
        Self {
            start,
            name: name.as_ref().to_string(),
            end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Location {
    line: usize,
    col: usize,
    loc: usize,
}

impl Location {
    pub fn new(loc: usize, s: &str) -> Self {
        let mut line = 1;
        let mut col = 1;
        for c in s.chars().take(loc) {
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        Self { loc, line, col }
    }
}

pub fn parse_string(s: &str) -> Result<Raytracer, ParseStringError> {
    let source_lines = &s.lines().collect::<Vec<_>>();

    match scene::SceneParser::new().parse(s) {
        Ok(scene) => match scene {
            Ok(raytracer) => Ok(raytracer),
            Err(scene_parse_error) => Err(ParseStringError::Many(
                scene_parse_error
                    .into_iter()
                    .map(|err| err.into_parse_string_error(s))
                    .collect(),
            )),
        },
        Err(parse_error) => Err(match parse_error {
            ParseError::InvalidToken { location } => {
                let start = Location::new(location, s);
                let end = Location::new(location + 1, s);

                ParseStringError::annotate(
                    source_lines,
                    &start,
                    Some(end).as_ref(),
                    "Invalid token".to_string(),
                )
            }
            ParseError::UnrecognizedEof {
                location: _,
                expected,
            } => ParseStringError::UnrecognizedEOF { expected },
            ParseError::UnrecognizedToken {
                token: (l, t, r),
                expected,
            } => ParseStringError::annotate(
                source_lines,
                &Location::new(l, s),
                Some(&Location::new(r, s)),
                format!(
                    "Unrecognized token '{t}'. Expected one of [ {} ]",
                    expected.join(", ")
                ),
            ),
            ParseError::ExtraToken { token: (l, t, r) } => ParseStringError::annotate(
                source_lines,
                &Location::new(l, s),
                Some(&Location::new(r, s)),
                t.to_string(),
            ),
            ParseError::User { error } => ParseStringError::User {
                error: error.to_string(),
            },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let s = r#"
            Camera {
                width: 512,
                height: 512,
                pos: (1.0,2.0,3.0),
                dir: (0.0,0.0,1),
            };

            Sphere {
                pos: (1.0,2.0,3.0),
                r: 0.1,
                material: {
                    color: (255, 0, 0),
                    template: "metal",
                    ambient: 0.1,
                }
            }

            Sphere {
                pos: (1.0,2.0,3.0),
                r: 0.1,
                material: {
                    color: (255, 0, 0),
                    ambient: 0.1,
                    lambert: 1.0,
                    specular: 0.1,
                }
            }

            Light {
                pos: (1,1,1),
                intensity: 1
            }
        "#
        .trim();
        let parsed = parse_string(&s);
        if !parsed.is_ok() {
            panic!("Expected Ok: {}", parsed.unwrap_err());
        }
    }

    #[test]
    fn multiple_keys_error() {
        let s = r#"
            Camera {
                pos: (1,1,1),
                dir: (1,1,1),
                pos: (1,1,1),
            }
        "#
        .trim();

        let parsed = parse_string(&s);
        assert!(parsed.is_err(), "{:#?}", parsed);
    }

    #[test]
    fn type_error() {
        let s = r#"
            Camera {
                pos: 1,
                dir: -1,
                width: 512,
                height: (1,1,1),
            }
        "#
        .trim();

        let parsed = parse_string(&s);
        assert!(parsed.is_err(), "{:#?}", parsed);
    }

    #[test]
    fn test_missing_key() {
        let s = r#"
            Camera {
                pos: (1,1,1),
                dir: (1,1,1),
                width: 512,
                height: 512,
            }

            Sphere {
                pos: (1,1,1),
                r: 1,
                material: {}
            }
        "#
        .trim();

        let parsed = parse_string(&s);
        assert!(parsed.is_err(), "{:#?}", parsed);
    }
}
