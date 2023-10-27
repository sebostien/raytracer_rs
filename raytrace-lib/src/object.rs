use crate::{
    material::Material,
    primitive::{Intersectable, Primitive},
    ray::Ray,
};

#[derive(Debug, Clone)]
pub struct Object {
    pub primitive: Primitive,
    pub material: Material,
}

impl Intersectable for Object {
    fn intersection(&self, ray: &Ray) -> Option<crate::primitive::Intersection> {
        self.primitive.intersection(ray)
    }
}
