use std::convert::TryFrom;
use serde_json::{
    from_value,
    Value as SerdeValue,
    Error as SerdeError};

use crate::utils;
use crate::raycast::{
    ray::Ray,
    vector::{Vector4, Vector3, UnitVector3},
    matrix::SquareMatrix4,
};


#[derive(Debug)]
pub struct Intersection {
    pub t: f32,
    pub incoming: UnitVector3,
    pub point: Vector3,
    pub normal: UnitVector3,
    pub material: Material,
}

pub trait Intersect {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection>;
}

// TODO It isn't exactly good to parse a matrix from some invented language
impl TryFrom<&str> for SquareMatrix4 {
    type Error = String; // TODO ParseError

    fn try_from(parse_string: &str) -> Result<Self, Self::Error> {
        let mat = parse_string.trim().split_terminator(';')
            .map(|s| {
                let s = s.trim();
                if s.starts_with("Translate") {
                    let v: Vec<f32> = s[s.find('e').unwrap() + 1..]
                        .split_ascii_whitespace()
                        .map(|q| q.trim().parse::<f32>().unwrap()) //TODO
                        .collect();
                    SquareMatrix4::translation(
                        Vector4 { x: v[0], y: v[1], z: v[2], w: 1.0 }
                    )
                } else if s.starts_with("RotX") {
                    let rads = utils::degs_to_rads(s[s.find('X').unwrap() + 1..]
                        .trim()
                        .parse::<f32>()
                        .unwrap()); //TODO
                    SquareMatrix4::rot_x(rads)
                } else if s.starts_with("RotY") {
                    let rads = utils::degs_to_rads(s[s.find('Y').unwrap() + 1..]
                        .trim()
                        .parse::<f32>()
                        .unwrap()); //TODO
                    SquareMatrix4::rot_y(rads)
                } else if s.starts_with("RotZ") {
                    let rads = utils::degs_to_rads(s[s.find('Z').unwrap() + 1..]
                        .trim()
                        .parse::<f32>()
                        .unwrap()); //TODO
                    SquareMatrix4::rot_z(rads)
                } else if s.starts_with("Scale") {
                    let v: Vec<f32> = s[s.find('e').unwrap() + 1..]
                        .split_ascii_whitespace()
                        .map(|q| q.trim().parse::<f32>().unwrap()) // TODO
                        .collect();
                    if let [a] = v[0..] {
                        // Scale all 3 dimensions the same
                        SquareMatrix4::scale(
                            Vector4 { x: a, y: a, z: a, w: a }
                        )
                    } else if let [x, y, z] = v[0..] {
                        // Scale each differently
                        SquareMatrix4::scale(
                            Vector4 { x, y, z, w: 1.0 }
                        )
                    } else {
                        // TODO return Err
                        panic!("Insufficient number of scaling values")
                    }
                } else {
                    // TODO return Err
                    panic!("Transformation not found '{}'", s)
                }
            })
            .fold(SquareMatrix4::identity(), |acc, x| &acc * &x);

        Ok(mat)
    }
}

#[derive(serde::Deserialize,Copy,Clone)]
#[derive(Debug)]
pub struct Material {
    pub color: color::Color,
    pub shininess: i32,
}
impl std::default::Default for Material {
    fn default() -> Self {
        Material { color: color::consts::GREY, shininess: 0 }
    }
}

pub struct Light {
    pub position: Vector3,
    pub _direction: Option<Vector3>,
    pub color: color::Color,
    pub intensity: f32,
}

impl TryFrom<SerdeValue> for Light {
    type Error = SerdeError;

    fn try_from(mut json: SerdeValue) -> Result<Self, SerdeError> {
        let color = from_value(json["color"].take())?;
        let intensity = from_value(json["intensity"].take())?;
        let position = from_value(json["position"].take())?;

        Ok(Light { position, _direction: None, color, intensity })
    }
}

pub mod color {
    use crate::raycast::{
        vector::{Vector3, Vector4},
        matrix::SquareMatrix4,
    };
    use image::Rgb;
    #[allow(dead_code)]
    pub mod consts {
        use super::*;
        pub const BLACK: Color = Color::new(0.0, 0.0, 0.0);
        pub const GREY:  Color = Color::new(0.5, 0.5, 0.5);
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
            // Conversion matrix values from:
            // www.scratchapixel.com/lessons/digital-imaging/colors/color-space
            const XYZ_TO_RGB: SquareMatrix4 = SquareMatrix4([
                [ 2.3706743, -0.9000405, -0.4706338, 0.0],
                [-0.5138850,  1.4253036,  0.0885814, 0.0],
                [ 0.0052982, -0.0146949,  1.0093968, 0.0],
                [       0.0,        0.0,        0.0, 1.0],
            ]);

            // XYZ color to RGB. Convert to Vector4 because matrix multip.
            // TODO/FIXME is the way `Color` is used actually according to XYZ?
            let Vector4 { x: r, y: g, z: b, w: _ } =
                &XYZ_TO_RGB * &Vector4::from_v3(c.0, 1.0);

            Rgb([
                (r.clamp(0.0, 1.0) * (u16::MAX as f32)) as u16,
                (g.clamp(0.0, 1.0) * (u16::MAX as f32)) as u16,
                (b.clamp(0.0, 1.0) * (u16::MAX as f32)) as u16,
            ])
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

