#[derive(serde::Deserialize, Copy,Clone,Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vector3 {
    pub fn normalized(self) -> UnitVector3 {
        UnitVector3::from(self)
    }
    pub fn length(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn cross(&self, other: &Self) -> Self {
        Vector3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x
        }
    }
}
impl std::ops::Mul<f32> for Vector3 {
    type Output = Self;
    fn mul(self, c: f32) -> Self::Output {
        Vector3 {
            x: self.x * c,
            y: self.y * c,
            z: self.z * c
        }
    }
}
impl std::ops::Mul<Vector3> for f32 {
    type Output = Vector3;
    fn mul(self, v: Vector3) -> Self::Output {
        v * self
    }
}
impl std::ops::Add<Vector3> for Vector3 {
    type Output = Vector3;
    fn add(self, other: Self) -> Self::Output {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z
        }
    }
}
impl std::ops::Sub<Vector3> for Vector3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z
        }
    }
}
impl std::ops::Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z
        }
    }
}
impl From<UnitVector3> for Vector3 {
    fn from(u: UnitVector3) -> Self {
        u.0
    }
}
impl From<&UnitVector3> for Vector3 {
    fn from(u: &UnitVector3) -> Self {
        u.0
    }
}

/// A 3D vector that is always normalized
#[derive(Copy,Clone,Debug)]
pub struct UnitVector3(Vector3);
impl UnitVector3 {
    pub fn reflect(&self, n: &Self) -> UnitVector3 {
        let v = self.0 - 2.0 * self.0.dot(&n.0) * n.0;
        UnitVector3::from(v)
    }
    pub fn dot(&self, other: &Self) -> f32 {
        self.0.dot(&other.0)
    }
    pub fn cross(&self, other: &Self) -> Self {
        UnitVector3::from(self.0.cross(&other.0))
    }
}

impl<'de> serde::Deserialize<'de> for UnitVector3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Vector3::deserialize(deserializer)?.normalized())
    }
}

impl std::ops::Neg for UnitVector3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        UnitVector3(-self.0)
    }
}
impl std::ops::Mul<f32> for UnitVector3 {
    type Output = Vector3;
    fn mul(self, c: f32) -> Self::Output {
        self.0 * c
    }
}
impl std::ops::Mul<UnitVector3> for f32 {
    type Output = Vector3;
    fn mul(self, v: UnitVector3) -> Self::Output {
        v.0 * self
    }
}
impl std::ops::Mul<f32> for &UnitVector3 {
    type Output = Vector3;
    fn mul(self, c: f32) -> Self::Output {
        self.0 * c
    }
}
impl std::ops::Mul<&UnitVector3> for f32 {
    type Output = Vector3;
    fn mul(self, v: &UnitVector3) -> Self::Output {
        v.0 * self
    }
}
impl From<Vector3> for UnitVector3 {
    fn from(v: Vector3) -> Self {
        let length = v.length();
        Self(
            Vector3 {
                x: v.x / length,
                y: v.y / length,
                z: v.z / length
            }
        )
    }
}

pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    pub fn from_v3(v: Vector3, w: f32) -> Self {
        let Vector3 { x, y, z } = v;
        Vector4 { x, y, z, w }
    }

    pub fn zero() -> Self {
        Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
        + self.z * other.z + self.w * other.w
    }

    pub fn xyz(&self) -> Vector3 {
        Vector3 { x: self.x, y: self.y, z: self.z }
    }
}
