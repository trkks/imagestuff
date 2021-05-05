#[derive(Copy,Clone,Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }
    pub fn normalized(&self) -> Self {
        let length = self.length();
        Vector3::new(self.x / length, self.y / length, self.z / length)
    }
    pub fn length(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }
    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn cross(lhs: Self, rhs: Self) -> Self {
        Vector3::new(
            lhs.y*rhs.z - lhs.z*rhs.y,
            lhs.z*rhs.x - lhs.x*rhs.z,
            lhs.x*rhs.y - lhs.y*rhs.x
        )
    }
}
impl std::ops::Mul<f32> for Vector3 {
    type Output = Self;
    fn mul(self, c: f32) -> Self::Output {
        Vector3::new(self.x * c, self.y * c, self.z * c)
    }
}
impl std::ops::Mul<Vector3> for f32 {
    type Output = Vector3;
    fn mul(self, v: Vector3) -> Self::Output {
        v * self
    }
}
impl std::ops::Add for Vector3 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Vector3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}
impl std::ops::Sub for Vector3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Vector3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}
impl std::ops::Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}
