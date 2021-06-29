use std::convert::TryFrom;
use crate::raycast::{
    vector3::{Vector3, UnitVector3},
    ray::Ray,
};

#[derive(Debug)]
pub struct Intersection {
    pub t: f32,
    pub point: Vector3,
    pub normal: UnitVector3,
    pub material: Material,
}

// TODO Make getting material a trait that returns some default if not
// specified in json

pub trait Intersect {
    //fn material(&self) -> Material;
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection>;
}

#[derive(serde::Deserialize,Copy,Clone)]
#[derive(Debug)]
pub struct Material {
    pub color: color::Color,
    pub shininess: i32,
}

pub struct Light {
    pub position: Vector3,
    pub _direction: Option<Vector3>,
    pub color: color::Color,
    pub intensity: f32,
}
impl TryFrom<&mut serde_json::Value> for Light {
    type Error = serde_json::Error;

    fn try_from(
        json: &mut serde_json::Value
    ) -> Result<Self,serde_json::Error> {
        let color = serde_json::from_value(json["color"].take())?;
        let intensity = serde_json::from_value(json["intensity"].take())?;
        let mut position = serde_json::from_value(json["position"].take())?;

        //position = &rot_y * &position;
        //position = &rot_x * &position;

        Ok(Light { position, _direction: None, color, intensity })
    }
}

pub mod color {
    use crate::raycast::{
        vector3::Vector3,
    };
    use image::Rgb;
    #[allow(dead_code)]
    pub mod consts {
        use super::*;
        pub const BLACK: Color = Color::new(0.0, 0.0, 0.0);
        pub const WHITE: Color = Color::new(1.0, 1.0, 1.0);
        pub const RED:   Color = Color::new(1.0, 0.0, 0.0);
        pub const GREEN: Color = Color::new(0.0, 1.0, 0.0);
        pub const BLUE:  Color = Color::new(0.0, 0.0, 1.0);
        pub const NEON_PINK: Color = Color::new(1.0, 0.43, 0.78);
    }

    /// Newtype to have some vector operations on a separate Color type
    #[derive(serde::Deserialize,Copy,Clone,Debug)]
    pub struct Color(Vector3);

    impl Color {
        pub const fn new(r: f32, g: f32, b: f32) -> Self {
            Color(Vector3 { x:r, y:g, z:b })
        }
    }

    impl From<Color> for Rgb<u16> { 
        fn from(c: Color) -> Self {
            let (r,g,b) = (c.0.x, c.0.y, c.0.z);
            Rgb(
                [ (r * (u16::MAX as f32)) as u16,
                  (g * (u16::MAX as f32)) as u16,
                  (b * (u16::MAX as f32)) as u16 ]
            )
        }
    }

    impl std::ops::Mul<f32> for Color {
        type Output = Self;
        fn mul(self, c: f32) -> Self::Output {
            Color(Vector3::mul(self.0, c))
        }
    }
    impl std::ops::Add for Color {
        type Output = Self;
        fn add(self, other: Color) -> Self::Output {
            Color(self.0 + other.0)
        }
    }
    impl std::ops::AddAssign<&Color> for Color {
        fn add_assign(&mut self, other: &Color) {
            self.0.x += other.0.x;
            self.0.y += other.0.y;
            self.0.z += other.0.z;
        }
    }
}
