use crate::raycast::vector3::Vector3;

pub struct SquareMatrix3([[f32;3];3]);
impl SquareMatrix3 {
    pub fn from(columns: [Vector3;3]) -> Self {
        Self([
            [columns[0].x, columns[1].x, columns[2].x],
            [columns[0].y, columns[1].y, columns[2].y],
            [columns[0].z, columns[1].z, columns[2].z],
        ])
    }    

    pub fn transposed(self) -> Self {
        Self([
            [self.0[0][0], self.0[1][0], self.0[2][0]],
            [self.0[0][1], self.0[1][1], self.0[2][1]],
            [self.0[0][2], self.0[1][2], self.0[2][2]],
        ])
    }

    pub fn col(&self, col: usize) -> Vector3 {
        Vector3 {
            x: self.0[0][col],
            y: self.0[1][col],
            z: self.0[2][col],
        }
    }

    pub fn determinant(&self) -> f32 {
        let det2x2 = |a,b,c,d| a * d - b * c;

        // Using the top row:
          self.0[0][0]
        * det2x2(self.0[1][1], self.0[1][2], self.0[2][1], self.0[2][2])
        - self.0[0][1]
        * det2x2(self.0[1][0], self.0[1][2], self.0[2][0], self.0[2][2])
        + self.0[0][2]
        * det2x2(self.0[1][0], self.0[1][1], self.0[2][0], self.0[2][1])
    }
}
