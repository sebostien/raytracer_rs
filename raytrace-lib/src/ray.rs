use crate::{
    color::Color,
    object::Object,
    primitive::{Intersectable, Intersection},
    vec3::Vec3,
};

/// A line that start from `origin` and moves in the direction of `dir`.
#[derive(Debug)]
pub struct Ray {
    /// The origin of the ray,
    pub origin: Vec3,
    /// Direction of the ray.
    /// Will always be a unit vector.
    dir: Vec3,
}

#[derive(Debug)]
pub struct RayHit {
    /// Color of the object which was hit.
    pub color: Color,
    /// The intersection point.
    pub intersection: Vec3,
    /// The normal of the reflection.
    pub normal: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            dir: direction.normalize(),
        }
    }

    pub fn direction(&self) -> &Vec3 {
        &self.dir
    }

    pub fn trace(&self, object: &Object) -> Option<RayHit> {
        object
            .intersection(self)
            .map(|Intersection { pos, normal }| RayHit {
                color: object.material.color,
                intersection: pos,
                normal,
            })
    }
}
