use serde::{Deserialize, Serialize};

use crate::{vec3::Vec3, UP_DIRECTION};

/// A 3d rotation matrix
#[derive(Debug, Deserialize, Serialize)]
pub struct Rotation {
    pub(crate) matrix: [[f64; 3]; 3],
}

impl From<Vec3> for Rotation {
    fn from(v: Vec3) -> Self {
        let v = v.normalize();
        let x_axis = UP_DIRECTION.cross(v).normalize();
        let Vec3 {
            x: yx,
            y: yy,
            z: yz,
        } = v.cross(x_axis);
        let Vec3 {
            x: xx,
            y: xy,
            z: xz,
        } = x_axis;

        Self {
            matrix: [[xx, yx, v.x], [xy, yy, v.y], [xz, yz, v.z]],
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn from_vec() {}
}
