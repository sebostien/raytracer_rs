use crate::vec3::Vec3;

#[derive(Debug)]
pub struct Light {
    pub pos: Vec3,
    pub intensity: f64,
}
