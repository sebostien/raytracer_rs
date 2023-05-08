//! A simple raytracer.

#![allow(unused)]

pub mod camera;
pub mod color;
pub mod light;
pub mod material;
pub mod object;
pub mod primitive;
pub mod ray;
pub mod rotation;
pub mod vec3;

pub use camera::Camera;
pub use color::Color;
pub use light::Light;
pub use material::Material;
pub use object::Object;
pub use vec3::Vec3;

use color::BLACK_COLOR;
use primitive::{Plane, Sphere};
use primitive::{Primitive, Triangle};
use ray::{Ray, RayHit};
use rotation::Rotation;

use std::f64::consts::PI;
use std::io::Write;

pub enum SceneObject {
    Camera(Camera),
    Primitive(Primitive),
    Light(Light),
}

/// Precision of comparisons.
pub const FLOAT_EPS: f64 = 0.000000001;

/// Distance in meters from camera to viewport.
const VIEWPORT_DISTANCE: f64 = 1.0;

/// The direction of “up”.
const UP_DIRECTION: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

#[derive(Debug)]
pub struct Raytracer {
    camera: Camera,
    world: Vec<Object>,
    lights: Vec<Light>,
    background_color: Color,
    recurse_depth: isize,
}

impl Raytracer {
    pub fn new(
        camera: Camera,
        world: Vec<Object>,
        lights: Vec<Light>,
        background_color: Color,
        recurse_depth: isize,
    ) -> Self {
        Self {
            camera,
            world,
            lights,
            background_color,
            recurse_depth,
        }
    }

    pub fn set_width(&mut self, width: u32) {
        self.camera.set_width(width);
    }

    pub fn set_height(&mut self, height: u32) {
        self.camera.set_height(height);
    }

    pub fn set_recurse_depth(&mut self, depth: usize) {
        self.recurse_depth = depth as isize;
    }
}

impl Raytracer {
    /// Return the position of any visible lights together with their intensity.
    fn trace_to_lights(&self, pos: Vec3) -> Vec<(Vec3, f64)> {
        let mut visible = vec![];

        for light in &self.lights {
            let ray = Ray::new(pos, pos - light.pos);
            for object in &self.world {
                if ray.trace(object).is_none() {
                    visible.push((light.pos, light.intensity));
                }
            }
        }

        visible
    }

    /// Lambertian reflection is the dot product of the surface normal
    /// and the light direction.
    /// <https://en.wikipedia.org/wiki/Lambertian_reflectance>
    fn lambertian(
        &self,
        object: &Object,
        intersection_pos: Vec3,
        intersection_normal: Vec3,
    ) -> f64 {
        if object.material.lambert <= 0.0 {
            return 0.0;
        }

        let mut brightness = 0.0;
        // TODO: Support multiple lights
        if let Some(&(light_pos, light_intensity)) = self.trace_to_lights(intersection_pos).first()
        {
            let contribution = intersection_pos
                .direction_to(light_pos)
                .normalize()
                .dot(intersection_normal)
                * light_intensity;

            if contribution > 0.0 {
                brightness += contribution;
            }
        }

        (brightness * object.material.lambert).min(1.0)
    }

    /// Reflect
    /// <https://en.wikipedia.org/wiki/Specular_reflection>
    fn specular(
        &self,
        object: &Object,
        intersection_pos: Vec3,
        intersection_normal: Vec3,
        depth: isize,
    ) -> Color {
        let mut color = BLACK_COLOR;

        if object.material.specular <= 0.0 {
            return color;
        }

        let reflected_dir = intersection_pos.reflect(intersection_normal);
        let new_ray = Ray::new(intersection_pos, reflected_dir);
        if let Some(reflected_color) = self.trace(new_ray, depth - 1) {
            color = color + reflected_color.scale(object.material.specular);
        }

        color
    }

    fn shading(
        &self,
        object: &Object,
        intersection_pos: Vec3,
        intersection_normal: Vec3,
        depth: isize,
    ) -> Color {
        let color = object.material.color.scale(self.lambertian(
            object,
            intersection_pos,
            intersection_normal,
        ));

        let color = color + self.specular(object, intersection_pos, intersection_normal, depth);

        color + object.material.color.scale(object.material.ambient)
    }

    /// Raycast from point with recursion level equal to `depth`.
    fn trace(&self, ray: Ray, depth: isize) -> Option<Color> {
        if depth <= 0 {
            return None;
        }

        let mut hit: Option<(f64, RayHit, &Object)> = None;

        for object in &self.world {
            if let Some(ray_hit) = ray.trace(object) {
                // Set minimum lambda as min of previous and this
                let dist = ray_hit.intersection.length_squared();
                if let Some((prev_dist, _, _)) = hit {
                    if dist < prev_dist {
                        hit = Some((dist, ray_hit, object));
                    }
                } else {
                    hit = Some((dist, ray_hit, object));
                }
            }
        }

        if let Some((_, ray_hit, object)) = hit {
            let color = self.shading(object, ray_hit.intersection, ray_hit.normal, depth - 1);
            Some(color)
        } else {
            None
        }
    }

    /// Returns the colors for each ray.
    /// Ordered by row then column.
    pub fn raycast(&self) -> Vec<Vec<Color>> {
        let (px, py) = self.camera.pixels();

        let mut image = vec![vec![self.background_color; px as usize]; py as usize];

        for (row, img_row) in image.iter_mut().enumerate() {
            let y = row as f64;
            for (col, img_cell) in img_row.iter_mut().enumerate() {
                let x = col as f64;
                let ray = self.camera.ray_from_pixel(x, y);

                if let Some(color) = self.trace(ray, self.recurse_depth) {
                    *img_cell = color;
                }
            }
        }

        image
    }
}
