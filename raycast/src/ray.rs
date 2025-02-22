use crate::vector::UnitVector3;

use super::vector::Vector3;

pub struct Ray {
    pub origin: Vector3,
    pub direction: UnitVector3,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vector3 {
        self.origin + self.direction * t
    }
}
