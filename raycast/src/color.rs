use super::math::vector::Vector3;

#[allow(dead_code)]
pub mod consts {
    use super::*;
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0);
    pub const GREY: Color = Color::new(0.5, 0.5, 0.5);
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0);
    pub const RED: Color = Color::new(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::new(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::new(0.0, 0.0, 1.0);
    pub const NEON_PINK: Color = Color::new(1.0, 0.43, 0.78);
}

/// Newtype to have some vector operations on a separate Color type
#[derive(Copy, Clone, Debug)]
pub struct Color(Vector3);

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Color(Vector3 { x: r, y: g, z: b })
    }
}

impl From<Color> for [u8; 3] {
    fn from(c: Color) -> Self {
        // Taking the square root applies "gamma 2"
        [
            (c.0.x.sqrt().clamp(0.0, 1.0) * (u8::MAX as f32 + 1.0)) as u8,
            (c.0.y.sqrt().clamp(0.0, 1.0) * (u8::MAX as f32 + 1.0)) as u8,
            (c.0.z.sqrt().clamp(0.0, 1.0) * (u8::MAX as f32 + 1.0)) as u8,
        ]
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Self;
    fn mul(self, c: f32) -> Self::Output {
        Color(Vector3::mul(self.0, c))
    }
}

impl std::ops::Mul<Color> for f32 {
    type Output = Color;
    fn mul(self, c: Color) -> Self::Output {
        Color(Vector3::mul(c.0, self))
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
        self.0 = self.0 + other.0;
    }
}

impl std::ops::MulAssign<f32> for Color {
    fn mul_assign(&mut self, other: f32) {
        self.0 = self.0 * other;
    }
}
