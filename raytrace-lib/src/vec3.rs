use crate::{rotation::Rotation, FLOAT_EPS};

/// Vector in 3d-space.
#[derive(Debug, Default, Clone, Copy)]
pub struct Vec3 {
    /// The x-component of the vector.
    pub x: f64,
    /// The y-component of the vector.
    pub y: f64,
    /// The z-component of the vector.
    pub z: f64,
}

impl Vec3 {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl Vec3 {
    #[must_use]
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[must_use]
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Returns the zero vector `{0,0,0}`.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    /// Returns the one vector `{1,1,1}`.
    #[must_use]
    pub fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }

    /// The length of the vector squared. Faster than [`Vec3::length`].
    ///
    /// [`Vec3::length`]: ./struct.Vec3.html#method.length
    #[must_use]
    pub fn length_squared(self) -> f64 {
        let x = self.x * self.x;
        let y = self.y * self.y;
        let z = self.z * self.z;
        x + y + z
    }

    /// The length of the vector.
    #[must_use]
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Normalize `self` into a unit vector.
    #[must_use]
    pub fn normalize(self) -> Self {
        let mag = self.length();
        Self {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }

    /// Returns a normalized vector that points from `self` to `other`.
    #[must_use]
    pub fn direction_to(self, other: Self) -> Self {
        (other - self).normalize()
    }

    /// Reflect `self` over a normal.
    /// Both `self` and the normal must be normalized.
    ///
    /// <https://en.wikipedia.org/wiki/Specular_reflection#Vector_formulation>
    #[must_use]
    pub fn reflect(self, normal: Self) -> Self {
        debug_assert!(self.is_unit() && normal.is_unit());
        self - 2.0 * normal * normal.dot(self)
    }

    /// Returns true if `self` as a unit vector.
    #[must_use]
    pub fn is_unit(self) -> bool {
        self.length() - 1.0 < FLOAT_EPS
    }

    /// Rotates the vector with the given rotation matrix.
    #[must_use]
    pub fn rotate(self, rot: &Rotation) -> Self {
        let Self { x, y, z } = self;
        let [[a, b, c], [d, e, f], [g, h, i]] = rot.matrix;
        Self::new(
            a * x + b * y + c * z,
            d * x + e * y + f * z,
            g * x + h * y + i * z,
        )
    }
}

impl std::ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: rhs.x * self,
            y: rhs.y * self,
            z: rhs.z * self,
        }
    }
}

impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < FLOAT_EPS
            && (self.y - other.y).abs() < FLOAT_EPS
            && (self.z - other.z).abs() < FLOAT_EPS
    }
}
