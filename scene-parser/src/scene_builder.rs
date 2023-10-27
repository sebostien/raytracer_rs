use crate::scene_object::{GlobalOptions, SceneObject};
use crate::SceneParseError;

use raytrace_lib::{Color, Light, Object, Raytracer};

pub struct SceneBuilder;

impl SceneBuilder {
    pub fn build(
        scene_objects: Vec<Result<SceneObject, SceneParseError>>,
    ) -> Result<(Vec<Object>, Vec<Light>, Raytracer), Vec<SceneParseError>> {
        let mut cameras = vec![];
        let mut objects = vec![];
        let mut lights = vec![];
        let mut errors = vec![];
        let mut options = GlobalOptions::default();

        for object in scene_objects {
            match object {
                Ok(object) => match object {
                    SceneObject::GlobalOptions(o) => {
                        options = o;
                    }
                    SceneObject::Camera(c) => cameras.push(c),
                    SceneObject::Object(p, m) => objects.push(Object {
                        primitive: p,
                        material: m,
                    }),
                    SceneObject::Light(l) => lights.push(l),
                },
                Err(obj_err) => {
                    errors.push(obj_err);
                }
            }
        }

        if cameras.len() != 1 {
            errors.push(SceneParseError::Custom {
                // TODO: Get location of (any) cameras
                start: 0,
                error: format!(
                    "There must be exactly one camera in a scene, found {}",
                    cameras.len()
                ),
                end: None, // TODO: location
            });
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Checked length above
        if let Some(camera) = cameras.pop() {
            Ok((
                objects,
                lights,
                Raytracer::new(camera, Color::new(0, 0, 0), options.recurse_depth.into()),
            ))
        } else {
            unreachable!()
        }
    }
}
