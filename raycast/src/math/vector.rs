pub trait Vector: Copy + Sized + Into<Vector3> {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn z(&self) -> f32;

    fn length(&self) -> f32 {
        (self.x().powi(2) + self.y().powi(2) + self.z().powi(2)).sqrt()
    }

    fn length_squared(&self) -> f32 {
        self.x().powi(2) + self.y().powi(2) + self.z().powi(2)
    }

    fn normalized(&self) -> UnitVector3 {
        let length = self.length();
        UnitVector3(Vector3 {
            x: self.x() / length,
            y: self.y() / length,
            z: self.z() / length,
        })
    }

    fn dot<T: Vector>(&self, other: &T) -> f32 {
        self.x() * other.x() + self.y() * other.y() + self.z() * other.z()
    }

    fn cross<T: Vector>(&self, other: &T) -> Vector3 {
        Vector3 {
            x: self.y() * other.z() - self.z() * other.y(),
            y: self.z() * other.x() - self.x() * other.z(),
            z: self.x() * other.y() - self.y() * other.x(),
        }
    }

    fn reflect<T: Vector>(&self, n: &T) -> UnitVector3 {
        let u = <T as Into<Vector3>>::into(*n) * 2.0 * self.dot(n);
        let v = <Self as Into<Vector3>>::into(*self) - u;
        v.normalized()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector for Vector3 {
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }
    fn z(&self) -> f32 {
        self.z
    }
}

impl std::ops::Mul<f32> for Vector3 {
    type Output = Vector3;
    fn mul(self, c: f32) -> Self::Output {
        Self::Output {
            x: self.x * c,
            y: self.y * c,
            z: self.z * c,
        }
    }
}

impl std::ops::Mul<Vector3> for f32 {
    type Output = Vector3;
    fn mul(self, v: Vector3) -> Self::Output {
        v * self
    }
}

impl std::ops::Div<f32> for Vector3 {
    type Output = Vector3;
    fn div(self, c: f32) -> Self::Output {
        Vector3 {
            x: self.x / c,
            y: self.y / c,
            z: self.z / c,
        }
    }
}

impl std::ops::Add<Vector3> for Vector3 {
    type Output = Vector3;
    fn add(self, other: Self) -> Self::Output {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl std::ops::Sub<Vector3> for Vector3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl std::ops::Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
/// A 3D vector that is always normalized
#[derive(Copy, Clone, Debug)]
pub struct UnitVector3(Vector3);

impl Vector for UnitVector3 {
    fn x(&self) -> f32 {
        self.0.x
    }
    fn y(&self) -> f32 {
        self.0.y
    }
    fn z(&self) -> f32 {
        self.0.z
    }
}

impl Into<Vector3> for UnitVector3 {
    fn into(self) -> Vector3 {
        self.0
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

impl std::ops::Div<f32> for UnitVector3 {
    type Output = Vector3;
    fn div(self, c: f32) -> Self::Output {
        self.0 / c
    }
}

impl std::ops::Add<Vector3> for UnitVector3 {
    type Output = Vector3;
    fn add(self, other: Vector3) -> Self::Output {
        self.0 + other
    }
}

impl std::ops::Sub<Vector3> for UnitVector3 {
    type Output = Vector3;
    fn sub(self, other: Vector3) -> Self::Output {
        self.0 - other
    }
}

impl std::ops::Neg for UnitVector3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        UnitVector3(-self.0)
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
        Vector4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn xyz(&self) -> Vector3 {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
