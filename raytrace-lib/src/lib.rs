//! A simple raytracer.

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
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
pub use vec3::Vec3;

use primitive::Primitive;
use ray::{Ray, RayHit};
use rotation::Rotation;

use std::sync::Arc;

pub enum SceneObject {
    Camera(Camera),
    Primitive(Primitive),
    Light(Light),
}

/// Precision of comparisons.
pub const FLOAT_EPS: f64 = 0.00000001;

/// The direction of “up”.
const UP_DIRECTION: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

#[derive(Debug)]
pub struct Raytracer {
    camera: Camera,
    recurse_depth: u32,
}

impl Raytracer {
    pub fn new(camera: Camera, recurse_depth: u32) -> Self {
        Self {
            camera,
            recurse_depth,
        }
    }

    pub fn set_width(&mut self, width: u32) {
        self.camera.set_width(width);
    }

    pub fn set_height(&mut self, height: u32) {
        self.camera.set_height(height);
    }

    pub fn set_recurse_depth(&mut self, depth: u32) {
        self.recurse_depth = depth;
    }
}

impl Raytracer {
    /// Return the position of any visible lights together with their intensity.
    fn trace_to_lights(world: &[Object], lights: &[Light], pos: Vec3) -> Vec<(Vec3, f64)> {
        let mut visible = vec![];

        for light in lights.iter() {
            let ray = Ray::new(pos, pos - light.pos);
            for object in world.iter() {
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
        world: &[Object],
        lights: &[Light],
        material: &Material,
        intersection_pos: Vec3,
        intersection_normal: Vec3,
    ) -> Color {
        if material.lambert.is_zero() {
            return Color::zero();
        }

        let mut brightness = 0.0;
        // TODO: Support multiple lights
        if let Some(&(light_pos, light_intensity)) =
            Self::trace_to_lights(world, lights, intersection_pos).first()
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

        material.lambert.scale(brightness.min(1.0))
    }

    /// Reflect
    /// <https://en.wikipedia.org/wiki/Specular_reflection>
    fn specular(
        world: &[Object],
        lights: &[Light],
        material: &Material,
        intersection_pos: Vec3,
        intersection_normal: Vec3,
        depth: u32,
    ) -> Color {
        if material.specular.is_zero() {
            return Color::zero();
        }

        let reflected_dir = intersection_pos.normalize().reflect(intersection_normal);
        let new_ray = Ray::new(intersection_pos, reflected_dir);

        Self::trace(world, lights, new_ray, depth.saturating_sub(1))
            .map(|c| c * material.specular)
            .unwrap_or(Color::zero())
    }

    fn shading(
        world: &[Object],
        lights: &[Light],
        material: &Material,
        intersection_pos: Vec3,
        intersection_normal: Vec3,
        depth: u32,
    ) -> Color {
        debug_assert!(intersection_normal.is_unit());

        let color = material.color
            * Self::lambertian(
                world,
                lights,
                material,
                intersection_pos,
                intersection_normal,
            );

        let color = color
            + Self::specular(
                world,
                lights,
                material,
                intersection_pos,
                intersection_normal,
                depth,
            );

        color + material.color * material.ambient
    }

    /// Raycast from point with recursion level equal to `depth`.
    fn trace(world: &[Object], lights: &[Light], ray: Ray, depth: u32) -> Option<Color> {
        if depth == 0 {
            return None;
        }

        let mut hit: Option<(f64, RayHit, &Object)> = None;

        for object in world.iter() {
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
            let color = Self::shading(
                world,
                lights,
                &object.material,
                ray_hit.intersection,
                ray_hit.normal,
                depth,
            );
            Some(color)
        } else {
            None
        }
    }
}

impl Raytracer {
    /// Returns the colors for each ray.
    /// Ordered by row then column.
    /// Traces using multiple threads.
    pub fn par_raycast(&self, world: Arc<[Object]>, lights: Arc<[Light]>) -> Vec<Vec<Color>> {
        let (px, py) = self.camera.pixels();

        let mut image = vec![vec![Color::zero(); px as usize]; py as usize];

        let px = f64::from(px);
        let py = f64::from(py);

        let depth = self.recurse_depth;

        image[..]
            .par_iter_mut()
            .enumerate()
            .for_each(|(row, img_row)| {
                img_row[..]
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(col, img_cell)| {
                        let py = py - (row as f64);
                        let px = (col as f64) - px / 2.0;

                        let ray = self.camera.ray_from_pixel(px, py);
                        if let Some(hit) = Self::trace(world.as_ref(), lights.as_ref(), ray, depth)
                        {
                            *img_cell = hit;
                        }
                    });
            });

        image
    }

    /// Returns the colors for each ray.
    /// Ordered by row then column.
    pub fn raycast(&self, world: &[Object], lights: &[Light]) -> Vec<Vec<Color>> {
        let (px, py) = self.camera.pixels();

        let mut image = vec![vec![Color::zero(); px as usize]; py as usize];

        let px = i64::from(px);
        let py = i64::from(py);

        for (row, y) in (-py..0).enumerate() {
            for (col, x) in (-px / 2..px / 2).enumerate() {
                let ray = self.camera.ray_from_pixel(x as f64, -y as f64);
                if let Some(hit) = Self::trace(world, lights, ray, self.recurse_depth) {
                    image[row][col] = hit;
                }
            }
        }

        image
    }
}
