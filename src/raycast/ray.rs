use crate::raycast::{
    vector::{Vector4, Vector3, UnitVector3},
    matrix::SquareMatrix4,
};

#[derive(Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: UnitVector3,
}
impl Ray {
    pub fn cast(&self, t: f32) -> Vector3 {
        self.origin + (self.direction * t)
    }

    pub fn with_transform(
        mut origin: Vector3,
        mut direction: UnitVector3,
        m: &SquareMatrix4,
    ) -> Self {
        origin = (m * &Vector4::from_v3(origin, 1.0)).xyz();

        // TODO Somewhere I got the feeling, that directions can't be
        // transformed like this
        direction = (m * &Vector4::from_v3(direction.into(), 0.0))
            .xyz()
            .normalized();

        Ray { origin, direction }
    }
}
