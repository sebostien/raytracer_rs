use serde::{Deserialize, Serialize};

use crate::vec3::Vec3;

#[derive(Debug, Deserialize, Serialize)]
pub struct Light {
    pub pos: Vec3,
    pub intensity: f64,
}
