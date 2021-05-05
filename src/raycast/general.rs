use crate::raycast::{
    vector3::Vector3,
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

pub struct Intersection {
    pub t: f32,
    pub point: Vector3,
    pub normal: Vector3,
    pub material: Material,
}
impl Intersection {
    pub fn new(t: f32, point: Vector3, normal: Vector3, material: Material) 
        -> Self 
    {
        Intersection {
            t,
            point,
            normal: normal.normalized(),
            material
        }
    }
}

pub trait Intersect {
    //fn material(&self) -> Material;
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection>;
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
    // TODO Find out if colors should not be represented as just 3D vectors 
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

// Generic name if this grows in the future
#[derive(serde::Deserialize,Copy,Clone)]
pub struct Material {
    pub color: color::Color,
}