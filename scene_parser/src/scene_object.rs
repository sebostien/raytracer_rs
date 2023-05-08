use crate::lit::SpannedLit;
use crate::options::Options;
use crate::{Ident, SceneParseError, DEFAULT_FOV};
use raytracer::primitive::{Plane, Primitive, Sphere, Triangle};
use raytracer::{Camera, Light, Material};

pub enum SceneObject {
    Camera(Camera),
    Object(Primitive, Material),
    Light(Light),
    GlobalOptions(GlobalOptions),
}

impl SceneObject {
    fn build_camera(ident: Ident, options: &mut Options) -> Result<Camera, SceneParseError> {
        let s = ident.start;
        let width = options.get("width", s)?.1.get_u32()?;
        let height = options.get("height", s)?.1.get_u32()?;
        let position = options.get("pos", s)?.1.get_vec3()?;
        let view_dir = options.get("dir", s)?.1.get_vec3()?;
        let fov = if let Ok(fov) = options.get("fov", s) {
            fov.1.get_double()?
        } else {
            DEFAULT_FOV
        };

        options.check_empty()?;
        Camera::new(width, height, position, view_dir, fov).map_err(|e| SceneParseError::Custom {
            start: ident.start,
            error: format!("{}", e),
            end: Some(ident.end),
        })
    }

    fn build_primitive(ident: &Ident, options: &mut Options) -> Result<Primitive, SceneParseError> {
        let start = ident.start;
        match ident.name.to_lowercase().as_str() {
            "sphere" => {
                let center = options.get("pos", start)?.1.get_vec3()?;
                let radius = options.get("r", start)?.1.get_double()?;
                options.check_empty()?;
                Ok(Primitive::Sphere(Sphere { center, radius }))
            }
            "triangle" => {
                let t1 = options.get("t1", start)?.1.get_vec3()?;
                let t2 = options.get("t2", start)?.1.get_vec3()?;
                let t3 = options.get("t3", start)?.1.get_vec3()?;
                options.check_empty()?;
                Ok(Primitive::Triangle(Triangle::new(t1, t2, t3)))
            }
            "plane" => {
                let point = options.get("point", start)?.1.get_vec3()?;
                let normal = options.get("normal", start)?.1.get_vec3()?;
                options.check_empty()?;
                Ok(Primitive::Plane(Plane::new(point, normal)))
            }
            _ => Err(SceneParseError::UnknownObject {
                start: ident.start,
                ident: ident.name.clone(),
                end: ident.end,
            }),
        }
    }

    fn build_material(ident: &Ident, options: &mut Options) -> Result<Material, SceneParseError> {
        let start = ident.start;

        let color = options.get("color", start)?.1.get_color()?;
        let lambert = options.get("lambert", start)?.1.get_double()?;
        let specular = options.get("specular", start)?.1.get_double()?;
        let ambient = options.get("ambient", start)?.1.get_double()?;

        options.check_empty()?;
        Ok(Material {
            color,
            lambert,
            specular,
            ambient,
        })
    }

    fn build_light(ident: Ident, options: &mut Options) -> Result<Light, SceneParseError> {
        let start = ident.start;
        let pos = options.get("pos", start)?.1.get_vec3()?;
        let intensity = options.get("intensity", start)?.1.get_double()?;

        options.check_empty()?;
        Ok(Light { pos, intensity })
    }

    fn build_global(ident: Ident, options: &mut Options) -> Result<GlobalOptions, SceneParseError> {
        let mut go = GlobalOptions::default();
        let start = ident.start;
        if let Ok((_, lit)) = options.get("recurse_depth", start) {
            go.recurse_depth = lit.get_u32()?;
        }
        options.check_empty()?;

        Ok(go)
    }

    pub fn new(
        ident: Ident,
        options: Vec<(Ident, SpannedLit)>,
    ) -> Result<SceneObject, SceneParseError> {
        let options = &mut Options::build(options)?;

        match ident.name.to_lowercase().as_str() {
            "global" => Ok(SceneObject::GlobalOptions(SceneObject::build_global(
                ident, options,
            )?)),
            "camera" => Ok(SceneObject::Camera(SceneObject::build_camera(
                ident, options,
            )?)),
            "light" => Ok(SceneObject::Light(SceneObject::build_light(
                ident, options,
            )?)),
            _ => {
                let material = options.get("material", ident.start);
                let prim = Self::build_primitive(&ident, options)?;
                let material = material?;
                let material_ident = material.0;
                let material: &mut Options = &mut material.1.try_into()?;
                let material = SceneObject::build_material(&material_ident, material)?;

                Ok(SceneObject::Object(prim, material))
            }
        }
    }
}

#[derive(Debug)]
pub struct GlobalOptions {
    pub recurse_depth: u32,
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self { recurse_depth: 5 }
    }
}
