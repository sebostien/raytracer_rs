//! A simple raytracer.

#![allow(unused)]

mod color;
mod light;
mod object;
mod primitive;
mod ray;
mod rotation;
mod vec3;

use image::{Rgb, RgbImage};
use primitive::Triangle;
use rotation::Rotation;
use serde::{Deserialize, Serialize};

use std::f64::consts::PI;

use crate::color::{Color, BLACK_COLOR};
use crate::light::Light;
use crate::object::Object;
use crate::primitive::{Plane, Primitive, Sphere};
use crate::ray::{Ray, RayHit};
use crate::vec3::Vec3;

/// Precision of comparisons.
pub const FLOAT_EPS: f64 = 0.000000001;

/// Distance in meters from camera to viewport.
const VIEWPORT_DISTANCE: f64 = 1.0;

/// The direction of "up".
const UP_DIRECTION: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

/// A grid in front of the camera.
///
/// The grid is 2 by 2 meter.
/// Top left: (-1,-1), Bottom right: (1,1)
#[derive(Debug, Deserialize, Serialize)]
struct Viewport {
    /// `width / height`
    aspect_ratio: f64,
    /// Number of horizontal pixels.
    pixels_x: usize,
    /// Number of vertical pixels.
    pixels_y: usize,
    /// The distance between two pixels in the x-direction.
    pixel_width: f64,
    /// The distance between two pixels in the y-direction.
    pixel_height: f64,
}

impl Viewport {
    /// Width and height are the number of pixels in
    /// the image which is used to calculate aspect ratio.
    pub fn new(width: u32, height: u32, fov: f64) -> Self {
        let w = width as f64;
        let h = height as f64;
        let aspect_ratio = w / h;

        Self {
            pixels_x: width as usize,
            pixels_y: height as usize,
            aspect_ratio,
            // The grid is 1m by 1m.
            // So we divide the grid evenly by the number of pixels.
            pixel_width: 1.0 / w,
            pixel_height: 1.0 / h / aspect_ratio,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Camera {
    /// The position of the camera.
    position: Vec3,
    /// Rotation of the camera.
    rotation: Rotation,
    /// The viewport to sends rays through.
    viewport: Viewport,
}

impl Camera {
    /// Create a new camera.
    ///
    /// # Arguments
    ///
    /// * `width`    - Number of horizontal pixels in the resulting frame
    /// * `height`   - Number of vertical pixels in the resulting frame
    /// * `position` - The position of the camera
    /// * `view_dir` - The direction that the camera looks in
    /// * `fov`      - Field of view in degrees [0, 180]
    pub fn new(width: u32, height: u32, position: Vec3, view_dir: Vec3, fov: f64) -> Self {
        let fov_rad = (fov / 2.0) * PI / 180.0;

        Self {
            position,
            rotation: view_dir.into(),
            viewport: Viewport::new(width, height, fov_rad),
        }
    }

    /// Returns a ray with origin from the cameras position
    /// and in the direction of the pixel.
    pub fn ray_from_pixel(&self, pixel_x: usize, pixel_y: usize) -> Ray {
        // Map pixels to range [-1, 1]
        let x = pixel_x as f64 * self.viewport.pixel_width - 0.5;
        let y = pixel_y as f64 * self.viewport.pixel_height - 0.5;

        let direction = Vec3::new(x, y, VIEWPORT_DISTANCE).rotate(&self.rotation);

        let origin = self.position;
        Ray::new(origin, direction)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Raytracer {
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
}
impl Raytracer {
    /// Return the positon of any visible lights togehter with their intensity.
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
        if object.lambert <= 0.0 {
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

        (brightness * object.lambert).min(1.0)
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

        if object.specular <= 0.0 {
            return color;
        }

        let reflected_dir = intersection_pos.reflect(intersection_normal);
        let new_ray = Ray::new(intersection_pos, reflected_dir);
        if let Some(reflected_color) = self.trace(new_ray, depth - 1) {
            color = color.add(&reflected_color.scale(object.specular));
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
        let color =
            object
                .color
                .scale(self.lambertian(object, intersection_pos, intersection_normal));

        let color = color.add(&self.specular(object, intersection_pos, intersection_normal, depth));

        color.add(&object.color.scale(object.ambient))
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
        let px = self.camera.viewport.pixels_x;
        let py = self.camera.viewport.pixels_y;

        let mut image = vec![vec![self.background_color; px]; py];

        for (row, img_row) in image.iter_mut().enumerate() {
            for (col, img_cell) in img_row.iter_mut().enumerate() {
                let ray = self.camera.ray_from_pixel(col, row);

                if let Some(color) = self.trace(ray, self.recurse_depth) {
                    *img_cell = color;
                }
            }
        }

        image
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let width = 1024;
    let height = 1024;

    let raytracer = Raytracer::new(
        Camera::new(
            width,
            height,
            Vec3 {
                x: 0.0,
                y: 3.0,
                z: -3.0,
            },
            Vec3 {
                x: 0.0,
                y: -0.2,
                z: 1.0,
            },
            45.0,
        ),
        vec![
            Object {
                primitive: Plane::new(Vec3::new(0.0, 0.0, 3.0), UP_DIRECTION).into(),
                color: Color::new(50, 200, 50),
                specular: 0.0,
                lambert: 1.0,
                ambient: 0.0,
            },
            Object {
                primitive: Triangle::new(
                    Vec3::new(-5.0, 10.0, 10.0),
                    Vec3::new(-5.0, 0.0, 10.0),
                    Vec3::new(5.0, 0.0, 10.0),
                )
                .into(),
                color: Color::new(0, 181, 226),
                specular: 1.0,
                lambert: 0.0,
                ambient: 0.1,
            },
            Object {
                primitive: Triangle::new(
                    Vec3::new(-5.0, 10.0, 10.0),
                    Vec3::new(5.0, 10.0, 10.0),
                    Vec3::new(5.0, 0.0, 10.0),
                )
                .into(),
                color: Color::new(0, 181, 226),
                specular: 1.0,
                lambert: 0.0,
                ambient: 0.1,
            },
            Object {
                primitive: Primitive::Plane(Plane::new(
                    Vec3::new(0.0, 0.0, -10.0),
                    Vec3::new(0.0, 0.0, -1.0),
                )),
                color: Color::new(32, 178, 170),
                specular: 0.0,
                lambert: 1.0,
                ambient: 0.0,
            },
            Object {
                primitive: Primitive::Plane(Plane::new(
                    Vec3::new(-5.0, 0.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                )),
                color: Color::new(0, 181, 226),
                specular: 0.0,
                lambert: 1.0,
                ambient: 0.0,
            },
            Object {
                primitive: Primitive::Plane(Plane::new(
                    Vec3::new(5.0, 0.0, 0.0),
                    Vec3::new(-1.0, 0.0, 0.0),
                )),
                color: Color::new(0, 181, 226),
                specular: 0.0,
                lambert: 1.0,
                ambient: 0.0,
            },
            Object {
                primitive: Plane::new(Vec3::new(0.0, 10.0, 0.0), -UP_DIRECTION).into(),
                color: Color::new(0, 181, 226),
                specular: 0.0,
                lambert: 1.0,
                ambient: 0.0,
            },
            Object {
                primitive: Sphere::new(Vec3::new(2.0, 0.0, 6.0), 0.5).into(),
                color: Color::new(255, 100, 100),
                specular: 0.0,
                lambert: 0.9,
                ambient: 0.1,
            },
            Object {
                primitive: Sphere::new(Vec3::new(0.0, 1.0, 6.0), 0.5).into(),
                color: Color::new(255, 100, 100),
                specular: 0.3,
                lambert: 0.5,
                ambient: 0.1,
            },
            Object {
                primitive: Sphere::new(Vec3::new(-2.0, 0.0, 6.0), 0.5).into(),
                color: Color::new(255, 100, 100),
                specular: 0.0,
                lambert: 0.9,
                ambient: 0.1,
            },
        ],
        vec![Light {
            pos: Vec3::new(-3.0, 8.0, 6.0),
            intensity: 0.5,
        }],
        Color::new(0, 0, 0),
        30,
    );

    let out = raytracer.raycast();

    let mut img = RgbImage::new(width, height);

    for (y, row) in out.iter().enumerate() {
        let y = height - 1 - y as u32;
        for (x, color) in row.iter().enumerate() {
            let x = x as u32;
            img.put_pixel(x, y, Rgb((*color).into()));
        }
    }

    img.save("raytrace.png")?;

    Ok(())
}
