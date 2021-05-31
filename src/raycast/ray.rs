use crate::raycast::vector3::{Vector3, UnitVector3};

#[derive(Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: UnitVector3,
}
impl Ray {
   pub fn cast(&self, t: f32) -> Vector3 {
        self.origin + (self.direction * t)
    }
}
