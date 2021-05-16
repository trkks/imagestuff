use crate::raycast::{
    vector3::Vector3,
    ray::Ray,
};

#[derive(serde::Deserialize,Debug)]
pub struct PerspectiveCamera {
    position: Vector3,
    direction: Vector3,
    horizontal: Vector3,
    up: Vector3,
    fov: f32,
    view_bounds: (f32, f32), // Range where 0 represents the CAMERA POSITION
}
impl PerspectiveCamera {
    pub fn new(position: Vector3, 
               direction: Vector3,
               up: Vector3,
               fov: f32,
                view_bounds: (f32, f32)) -> Self {
        let direction  = direction.normalized();
        let horizontal = Vector3::cross(direction, up).normalized(); 
        let up         = Vector3::cross(horizontal, direction).normalized(); 

        PerspectiveCamera {
            position,
            direction,
            horizontal,
            up,
            fov,
            view_bounds
        }
    }
    /// This is used to convert from json-conversion, which has fov in degrees
    /// (and a serde_json-forced value for the calculated field "horizontal")
    pub fn from(source: PerspectiveCamera) -> Self {
        Self::new(source.position,
                  source.direction,
                  source.up,
                  source.fov * (std::f32::consts::PI/180.0),
                  source.view_bounds)
    }
    pub fn shoot_at(&self, x: f32, y: f32) -> Ray {
        // NOTE This assumes that x and y have been scaled into [-1, 1]
        let z = 1.0 / f32::tan(self.fov / 2.0);

        // Generate ray from camera to the image plane
        let ray_direction =   x * self.horizontal
                            + y * self.up
                            + z * self.direction
                            - self.position;

        Ray::new(self.position, ray_direction)
    }
}


