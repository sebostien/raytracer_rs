use std::collections::HashMap;

use crate::{lit::SpannedLit, Ident, SceneParseError};

pub struct Options(HashMap<String, (Ident, SpannedLit)>);

impl Options {
    pub fn build(options: Vec<(Ident, SpannedLit)>) -> Result<Self, SceneParseError> {
        let mut map = HashMap::new();

        for (k, v) in options {
            if map.insert(k.name.to_lowercase(), (k.clone(), v)).is_some() {
                return Err(SceneParseError::DuplicateKey {
                    start: k.start,
                    key: k.name,
                });
            }
        }

        Ok(Self(map))
    }

    pub fn get(
        &mut self,
        name: &str,
        ident_location: usize,
    ) -> Result<(Ident, SpannedLit), SceneParseError> {
        if let Some(opt) = self.0.remove(name) {
            Ok(opt)
        } else {
            Err(SceneParseError::MissingOption {
                start: ident_location,
                name: name.to_string(),
            })
        }
    }

    pub fn check_empty(&mut self) -> Result<(), SceneParseError> {
        let idents: Vec<_> = self.0.drain().map(|(_, (ident, _))| ident).collect();

        if idents.is_empty() {
            Ok(())
        } else {
            Err(SceneParseError::UnknownOptions { idents })
        }
    }
}
