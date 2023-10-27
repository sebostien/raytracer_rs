use crate::{ray::Ray, vec3::Vec3, FLOAT_EPS};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Intersection {
    /// The position of the intersection.
    pub pos: Vec3,
    /// The normal at the intersection point.
    pub normal: Vec3,
}

pub trait Intersectable {
    /// Check if the ray intersects the intersectable.
    fn intersection(&self, ray: &Ray) -> Option<Intersection>;
}

#[derive(Debug, Clone, Copy)]
pub enum Primitive {
    Sphere(Sphere),
    Triangle(Triangle),
    Plane(Plane),
}

impl Intersectable for Primitive {
    fn intersection(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            Self::Sphere(s) => s.intersection(ray),
            Self::Triangle(s) => s.intersection(ray),
            Self::Plane(s) => s.intersection(ray),
        }
    }
}

/// An infinite plane described by a point and a normal.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    point: Vec3,
    normal: Vec3,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3) -> Self {
        Self{
            point,
            normal: normal.normalize(),
        }
    }

    /// Return a plane from the Cartesian equation `ax + by + cz + d = 0`.
    #[allow(unused)]
    fn from_cartesian(a: f64, b: f64, c: f64, d: f64) -> Self {
        // ax + by + cz + d = 0    (x=0, y=0)
        // z = - d / c
        Self::new(Vec3::new(0.0, 0.0, -d / c), Vec3::new(a, b, c))
    }
}

impl From<Plane> for Primitive {
    fn from(value: Plane) -> Self {
        Self::Plane(value)
    }
}

impl Intersectable for Plane {
    fn intersection(&self, ray: &Ray) -> Option<Intersection> {
        // Implemented from the wikipedia page about line-plane intersections.
        // <https://en.wikipedia.org/wiki/Line%E2%80%93plane_intersection#Algebraic_form>

        let normal = self.normal;
        let plane_point = self.point;
        let ray_origin = ray.origin;
        let ray_dir = ray.direction();

        let dir_dot_normal = ray_dir.dot(normal);

        // Line and plane are parallel
        if dir_dot_normal.abs() < FLOAT_EPS {
            return None;
        }

        let d = (plane_point - ray_origin).dot(normal.to_owned()) / dir_dot_normal;

        // Intersection behind the ray origin
        if d < FLOAT_EPS {
            return None;
        }

        Some(Intersection {
            pos: ray_origin + (*ray_dir * d),
            normal,
        })
    }
}

/// A triangle in 3d-space.
///
/// The three vectors makes up each corner of the triangle.
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub t1: Vec3,
    pub t2: Vec3,
    pub t3: Vec3,
    // The normal of the triangles plane.
    pub normal: Vec3,
    // Line from `t1` to `t2`.
    pub l12: Vec3,
    // Line from `t1` to `t3`.
    pub l13: Vec3,
}

impl Triangle {
    pub fn new(t1: Vec3, t2: Vec3, t3: Vec3) -> Self {
        // Perpendicular to the plane of the triangle
        let l12 = t2 - t1;
        let l13 = t3 - t1;
        let normal = l12.cross(l13);
        Self {
            t1,
            t2,
            t3,
            normal,
            l12,
            l13,
        }
    }
}

impl From<Triangle> for Primitive {
    fn from(value: Triangle) -> Self {
        Self::Triangle(value)
    }
}

impl Intersectable for Triangle {
    fn intersection(&self, ray: &Ray) -> Option<Intersection> {
        // The Möller–Trumbore intersection algorithm.
        // <https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm>
        let ray_dir = *ray.direction();
        let ray_origin = ray.origin;

        let h = ray_dir.cross(self.l13);
        let a = self.l12.dot(h);

        if (-FLOAT_EPS..FLOAT_EPS).contains(&a) {
            // The ray is parallel to this triangle.
            return None;
        }

        let f = 1.0 / a;
        let s = ray_origin - self.t1;
        let u = f * s.dot(h);

        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = s.cross(self.l12);
        let v = f * ray_dir.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        // Distance along the ray travelled
        let distance = f * self.l13.dot(q);

        // Intersection behind ray origin
        if distance < FLOAT_EPS {
            return None;
        }

        let out_intersection_point = ray_origin + ray_dir * distance;
        Some(Intersection {
            pos: out_intersection_point,
            normal: self.normal,
        })
    }
}

/// A triangle in 3d-space.
///
/// The three vectors makes up each corner of the triangle.
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64) -> Self {
        Self { center, radius }
    }
}

impl From<Sphere> for Primitive {
    fn from(value: Sphere) -> Self {
        Self::Sphere(value)
    }
}

impl Intersectable for Sphere {
    fn intersection(&self, ray: &Ray) -> Option<Intersection> {
        // From: <https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection.html>
        // Where the direction of the ray is a unit vector.

        // Sanity check. Ray should guarantee this.
        debug_assert!(
            ray.direction().is_unit(),
            "Expected unit vector for ray direction. Got vector '{:?}' with length '{}'",
            ray,
            ray.direction().length()
        );

        let dir = *ray.direction();
        let r = self.radius;

        let l = ray.origin - self.center;
        let a = dir.dot(dir);
        let b = dir.dot(l) * 2.0;
        let c = l.dot(l) - r * r;

        let discr = b * b - 4.0 * a * c;

        let (t0, t1) = match (discr < -FLOAT_EPS, discr < FLOAT_EPS) {
            (true, _) => {
                // < 0    No intersection
                return None;
            }
            (_, true) => {
                // == 0   One intersection
                let t = -0.5 * b / a;
                (t, t)
            }
            _ => {
                // > 0    Two intersections
                let x = if b > 0.0 {
                    b + discr.sqrt()
                } else {
                    b - discr.sqrt()
                };

                let q = -0.5 * x;
                (q / a, c / q)
            }
        };

        // Minimum but not negative
        let t = match (t0 < 0.0, t1 < 0.0) {
            (true, true) => {
                return None;
            }
            (true, _) => t1,
            (_, true) => t0,
            _ => t0.min(t1),
        };

        let pos = ray.origin + dir * t;
        let normal = pos - self.center;

        Some(Intersection { pos, normal })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3::Vec3;

    #[test]
    fn triangle_intersect() {
        let ray = Ray::new(Vec3::new(-1.5, -0.5, -1.0), Vec3::new(1.0, 1.0, 1.0));

        let tri = Triangle::new(
            Vec3::new(-3.0, -2.0, 1.0),
            Vec3::new(3.0, 2.0, 1.0),
            Vec3::new(-3.0, 2.0, -2.0),
        );
        assert_eq!(
            tri.intersection(&ray).unwrap().pos,
            Vec3::new(-0.2, 0.8, 0.3)
        );

        let tri = Triangle::new(
            Vec3::new(-1.5, 0.5, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 0.0),
        );
        assert!(tri.intersection(&ray).is_none());
    }

    #[test]
    fn sphere_intersect() {
        let sphere = Sphere::new(Vec3::new(-7.04, 5.16, 2.0), 1.5);
        let ray = Ray::new(Vec3::new(-0.19, 1.82, 1.0), Vec3::new(-2.0, 1.31, 0.48));

        assert_eq!(
            sphere.intersection(&ray).unwrap().pos,
            Vec3::new(-5.581611341953535, 5.351505428979565, 2.2939867220688486)
        );
    }

    #[test]
    fn plane_parallel() {
        let p = Plane::from_cartesian(-3.0, -2.0, 1.0, -4.0);
        let ray = Ray::new(Vec3::new(2.0, -3.0, 4.0), Vec3::new(2.0, -4.0, -2.0));
        assert_eq!(p.intersection(&ray), None);

        let p = Plane::from_cartesian(2.0, -3.0, 5.0, -10.0);
        let ray = Ray::new(Vec3::new(-1.0, 7.0, 4.0), Vec3::new(1.0, -7.0, -4.6));
        assert_eq!(p.intersection(&ray), None);
    }

    #[test]
    fn plane_intersect() {
        let p = Plane::from_cartesian(2.0, 1.0, -1.0, -45.0);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(3.0, 3.0, 4.0));
        assert_eq!(
            p.intersection(&ray),
            Some(Intersection {
                pos: Vec3::new(27.0, 27.0, 36.0),
                normal: Vec3::new(2.0, 1.0, -1.0).normalize()
            })
        );

        let p = Plane::from_cartesian(-2.0, 6.0, -3.0, -35.0);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(8.0, 8.0, 4.0));
        assert_eq!(
            p.intersection(&ray),
            Some(Intersection {
                pos: Vec3::new(14.0, 14.0, 7.0),
                normal: Vec3::new(-2.0, 6.0, -3.0).normalize()
            })
        );

        let p = Plane::from_cartesian(2.0, -1.0, 3.0, -15.0);
        let ray = Ray::new(Vec3::new(4.0, -1.0, 3.0), Vec3::new(1.0, 8.0, -2.0));
        assert_eq!(
            p.intersection(&ray),
            Some(Intersection {
                pos: Vec3::new(4.25, 1.0, 2.5),
                normal: Vec3::new(2.0, -1.0, 3.0).normalize()
            })
        );

        let p = Plane::from_cartesian(2.0, -3.0, 1.0, -14.0);
        let ray = Ray::new(Vec3::new(1.0, 0.0, -1.0), Vec3::new(2.0, -3.0, 0.0));
        assert_eq!(
            p.intersection(&ray),
            Some(Intersection {
                pos: Vec3::new(3.0, -3.0, -1.0),
                normal: Vec3::new(2.0, -3.0, 1.0).normalize()
            })
        );

        let p = Plane::from_cartesian(-5.0, 4.0, -1.0, 4.0);
        let ray = Ray::new(Vec3::new(1.0, -2.0, 1.0), Vec3::new(-3.0, 3.0, 3.0));
        assert_eq!(
            p.intersection(&ray),
            Some(Intersection {
                pos: Vec3::new(-0.25, -0.75, 2.25),
                normal: Vec3::new(-5.0, 4.0, -1.0).normalize()
            })
        );
    }
}
