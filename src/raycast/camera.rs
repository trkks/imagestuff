use std::convert::TryFrom;
use crate::raycast::{
    vector3::{Vector3, UnitVector3},
    ray::Ray,
};

#[derive(Debug)]
pub struct PerspectiveCamera {
    position: Vector3,
    direction: UnitVector3,
    horizontal: UnitVector3,
    up: UnitVector3,
    fov: f32,
    view_bounds: (f32, f32), // Range where 0 represents the CAMERA POSITION
}
impl PerspectiveCamera {
    pub fn new(
        position: Vector3,
        direction: UnitVector3,
        up: UnitVector3,
        fov: f32,
        view_bounds: (f32, f32)
    ) -> Self {
        // Create a basis for camera
        let horizontal = direction.cross(&up);
        let up         = horizontal.cross(&direction);

        PerspectiveCamera {
            position,
            direction,
            horizontal,
            up,
            fov,
            view_bounds
        }
    }

    pub fn shoot_at(&self, x: f32, y: f32) -> Ray {
        // NOTE This assumes that x and y have been scaled into [-1, 1]
        let z = 1.0 / f32::tan(self.fov / 2.0);

        // Generate ray from camera to the image plane
        let ray_direction =   x * self.horizontal
                            + y * self.up
                            + z * self.direction
                            - self.position;

        Ray { origin: self.position, direction: ray_direction.normalized() }
    }
}

impl TryFrom<serde_json::Value> for PerspectiveCamera {
    type Error = serde_json::Error;
    fn try_from(
        mut json: serde_json::Value
    ) -> Result<Self, serde_json::Error> {
        const TO_RADS: f32 = std::f32::consts::PI / 180.0;
        let position = serde_json::from_value(json["position"].take())?;
        let direction = {
            let v: Vector3 = serde_json::from_value(json["direction"].take())?;
            v.normalized()
        };
        let up = {
            let v: Vector3 = serde_json::from_value(json["up"].take())?;
            v.normalized()
        };
        let fov = serde_json::from_value::<f32>(json["fov"].take())? * TO_RADS;
        let view_bounds = serde_json::from_value(json["view_bounds"].take())?;

        Ok(Self::new(position, direction, up, fov, view_bounds))
    }
}
