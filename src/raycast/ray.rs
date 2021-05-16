use crate::raycast::vector3::Vector3;

pub struct Ray {
    origin: Vector3,
    direction: Vector3,
}

impl Ray {
    pub fn new(origin: Vector3, direction: Vector3) -> Self {
        // TODO Make normalized direction an invariant of Ray
        Ray { origin, direction: direction }   
    }
    pub fn origin(&self) -> Vector3 { self.origin }
    pub fn direction(&self) -> Vector3 { self.direction.normalized() }
    pub fn cast(&self, t: f32) -> Vector3 {
        self.origin + (self.direction() * t)
    }
}
