use serde::{Deserialize, Serialize};

use crate::{
    color::Color,
    primitive::{Intersectable, Primitive},
    ray::Ray,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Object {
    pub primitive: Primitive,
    pub color: Color,
    /// Specular reflection defines how much of light the object reflects.
    /// Should be in range \[0,1\].
    /// <https://en.wikipedia.org/wiki/Specular_reflection>
    pub specular: f64,
    /// Lamberterian reflectance defines how "matte" the object appears.
    /// Should be in range \[0,1\].
    /// <https://en.wikipedia.org/wiki/Lambertian_reflectance>
    pub lambert: f64,
    /// Ambient lighting defines how strong the "base light" should be interpreted.
    /// Should be in range \[0,1\].
    /// <https://en.wikipedia.org/wiki/Shading#Ambient_lighting>
    pub ambient: f64,
}

impl Intersectable for Object {
    fn intersection(&self, ray: &Ray) -> Option<crate::primitive::Intersection> {
        self.primitive.intersection(ray)
    }
}
