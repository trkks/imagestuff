use std::convert::TryInto;
use crate::raycast::vector::Vector4;

#[derive(Debug)]
pub struct SquareMatrix4(pub [[f32;4];4]);
impl SquareMatrix4 {
    pub fn determinant(&self) -> f32 {
        let det2x2 = |a,b, c,d| a * d - b * c;
        // Using the top row:
        let det3x3 = |a,b,c, d,e,f, g,h,i|
              a * det2x2(e, f, h, i)
            - b * det2x2(d, f, g, i)
            + c * det2x2(d, e, g, h);

        // Destructure the matrix
        let [ [a, b, c, d],
              [e, f, g, h],
              [i, j, k, l],
              [m, n, o, p] ] = self.0;
        // Using the top row:
          a * det3x3(f,g,h, j,k,l, n,o,p)
        - b * det3x3(e,g,h, i,k,l, m,o,p)
        + c * det3x3(e,f,h, i,j,l, m,n,p)
        - d * det3x3(e,f,g, i,j,k, m,n,o)
    }

    pub fn inversed(&self) -> Self {
        let det2x2 = |a,b, c,d| a * d - b * c;
        let det3x3 = |a,b,c, d,e,f, g,h,i|
              a * det2x2(e, f, h, i)
            - b * det2x2(d, f, g, i)
            + c * det2x2(d, e, g, h);

        // Destructure the matrix
        let [ [a, b, c, d],
              [e, f, g, h],
              [i, j, k, l],
              [m, n, o, p] ] = self.0;

        // Minors and cofactors
        let mut mat = SquareMatrix4([
            [
              det3x3(f,g,h, j,k,l, n,o,p),
             -det3x3(e,g,h, i,k,l, m,o,p),
              det3x3(e,f,h, i,j,l, m,n,p),
             -det3x3(e,f,g, i,j,k, m,n,o),
            ],
            [
             -det3x3(b,c,d, j,k,l, n,o,p),
              det3x3(a,c,d, i,k,l, m,o,p),
             -det3x3(a,b,d, i,j,l, m,n,p),
              det3x3(a,b,c, i,j,k, m,n,o),
            ],
            [
              det3x3(b,c,d, f,g,h, n,o,p),
             -det3x3(a,c,d, e,g,h, m,o,p),
              det3x3(a,b,d, e,f,h, m,n,p),
             -det3x3(a,b,c, e,f,g, m,n,o),
            ],
            [
             -det3x3(b,c,d, f,g,h, j,k,l),
              det3x3(a,c,d, e,g,h, i,k,l),
             -det3x3(a,b,d, e,f,h, i,j,l),
              det3x3(a,b,c, e,f,g, i,j,k),
            ],
        ]);

        // Adjugate
        mat.transpose();

        // Find determinant based on the matrix of minors
        let det =
              a * mat.0[0][0]
            + b * mat.0[0][1]
            + c * mat.0[0][2]
            + d * mat.0[0][3];

        let x = 1.0 / det;

        // "Divide" original matrix by determinant
        let arrs: Vec<f32> =
            mat.0.iter().flatten().map(|e| e * x).collect();

        SquareMatrix4([
            arrs[ 0.. 4].try_into().unwrap(),
            arrs[ 4.. 8].try_into().unwrap(),
            arrs[ 8..12].try_into().unwrap(),
            arrs[12..16].try_into().unwrap(),
        ])
    }

    pub fn transpose(&mut self) {
        self.0 = [
            [self.0[0][0], self.0[1][0], self.0[2][0], self.0[3][0]],
            [self.0[0][1], self.0[1][1], self.0[2][1], self.0[3][1]],
            [self.0[0][2], self.0[1][2], self.0[2][2], self.0[3][2]],
            [self.0[0][3], self.0[1][3], self.0[2][3], self.0[3][3]],
        ];
    }

    pub fn transposed(&self) -> Self {
        Self([
            [self.0[0][0], self.0[1][0], self.0[2][0], self.0[3][0]],
            [self.0[0][1], self.0[1][1], self.0[2][1], self.0[3][1]],
            [self.0[0][2], self.0[1][2], self.0[2][2], self.0[3][2]],
            [self.0[0][3], self.0[1][3], self.0[2][3], self.0[3][3]],
        ])
    }

    pub fn identity() -> Self {
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn translation(v: Vector4) -> Self {
        Self([
            [1.0, 0.0, 0.0, v.x],
            [0.0, 1.0, 0.0, v.y],
            [0.0, 0.0, 1.0, v.z],
            [0.0, 0.0, 0.0, v.w],
        ])
    }

    pub fn scale(v: Vector4) -> Self {
        Self([
            [v.x, 0.0, 0.0, 0.0],
            [0.0, v.y, 0.0, 0.0],
            [0.0, 0.0, v.z, 0.0],
            [0.0, 0.0, 0.0, v.w],
        ])
    }

    pub fn rot_x(radians: f32) -> Self {
        Self([
            [ 1.0,           0.0,            0.0, 0.0],
            [ 0.0, radians.cos(), -radians.sin(), 0.0],
            [ 0.0, radians.sin(),  radians.cos(), 0.0],
            [ 0.0,           0.0,            0.0, 1.0]
        ])
    }

    pub fn rot_y(radians: f32) -> Self {
        Self([
            [ radians.cos(), 0.0, radians.sin(), 0.0],
            [           0.0, 1.0,           0.0, 0.0],
            [-radians.sin(), 0.0, radians.cos(), 0.0],
            [           0.0, 0.0,           0.0, 1.0]
        ])
    }

    pub fn rot_z(radians: f32) -> Self {
        Self([
            [ radians.cos(), -radians.sin(), 0.0, 0.0],
            [ radians.sin(),  radians.cos(), 0.0, 0.0],
            [           0.0,            0.0, 1.0, 0.0],
            [           0.0,            0.0, 0.0, 1.0]
        ])
    }
}

impl std::ops::Mul for &SquareMatrix4 {
    type Output = SquareMatrix4;
    fn mul(self, rhs: &SquareMatrix4) -> Self::Output {
        // Left hand side
        let [ [la, lb, lc, ld],
              [le, lf, lg, lh],
              [li, lj, lk, ll],
              [lm, ln, lo, lp] ] = self.0;
        // Right hand side
        let [ [ra, rb, rc, rd],
              [re, rf, rg, rh],
              [ri, rj, rk, rl],
              [rm, rn, ro, rp] ] = rhs.0;

        SquareMatrix4([
            [
                la * ra + lb * re + lc * ri + ld * rm,
                la * rb + lb * rf + lc * rj + ld * rn,
                la * rc + lb * rg + lc * rk + ld * ro,
                la * rd + lb * rh + lc * rl + ld * rp,
            ],
            [
                le * ra + lf * re + lg * ri + lh * rm,
                le * rb + lf * rf + lg * rj + lh * rn,
                le * rc + lf * rg + lg * rk + lh * ro,
                le * rd + lf * rh + lg * rl + lh * rp,
            ],
            [
                li * ra + lj * re + lk * ri + ll * rm,
                li * rb + lj * rf + lk * rj + ll * rn,
                li * rc + lj * rg + lk * rk + ll * ro,
                li * rd + lj * rh + lk * rl + ll * rp,
            ],
            [
                lm * ra + ln * re + lo * ri + lp * rm,
                lm * rb + ln * rf + lo * rj + lp * rn,
                lm * rc + ln * rg + lo * rk + lp * ro,
                lm * rd + ln * rh + lo * rl + lp * rp,
            ],
        ])
    }
}

impl std::ops::Mul<&Vector4> for &SquareMatrix4 {
    type Output = Vector4;
    fn mul(self, v: &Vector4) -> Self::Output {
        let x = self.0[0][0] * v.x
            + self.0[0][1] * v.y
            + self.0[0][2] * v.z
            + self.0[0][3] * v.w;

        let y = self.0[1][0] * v.x
            + self.0[1][1] * v.y
            + self.0[1][2] * v.z
            + self.0[1][3] * v.w;

        let z = self.0[2][0] * v.x
            + self.0[2][1] * v.y
            + self.0[2][2] * v.z
            + self.0[2][3] * v.w;

        let w = self.0[3][0] * v.x
            + self.0[3][1] * v.y
            + self.0[3][2] * v.z
            + self.0[3][3] * v.w;

        Vector4 { x, y, z, w }
    }
}

impl std::fmt::Display for SquareMatrix4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>)
    -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:?}\n{:?}\n{:?}\n{:?}",
            self.0[0],
            self.0[1],
            self.0[2],
            self.0[3]
        )
    }
}
