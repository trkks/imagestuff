use super::vector::Vector4;
use std::convert::TryInto;

fn det2x2(a: f32, b: f32, c: f32, d: f32) -> f32 {
    a * d - b * c
}

#[allow(clippy::too_many_arguments)]
fn det3x3(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32, g: f32, h: f32, i: f32) -> f32 {
    // Using the top row:
    a * det2x2(e, f, h, i) - b * det2x2(d, f, g, i) + c * det2x2(d, e, g, h)
}

#[derive(Debug)]
pub struct SquareMatrix4(pub [[f32; 4]; 4]);
impl SquareMatrix4 {
    pub fn determinant(&self) -> f32 {
        // Destructure the matrix
        let [[a, b, c, d], [e, f, g, h], [i, j, k, l], [m, n, o, p]] = self.0;
        // Using the top row:
        a * det3x3(f, g, h, j, k, l, n, o, p) - b * det3x3(e, g, h, i, k, l, m, o, p)
            + c * det3x3(e, f, h, i, j, l, m, n, p)
            - d * det3x3(e, f, g, i, j, k, m, n, o)
    }

    pub fn inversed(&self) -> Option<Self> {
        // Destructure the matrix
        let [[a0, b0, c0, d0], [e, f, g, h], [i, j, k, l], [m, n, o, p]] = self.0;

        // Minors
        let [[a, b, c, d], [e, f, g, h], [i, j, k, l], [m, n, o, p]] = [
            [
                det3x3(f, g, h, j, k, l, n, o, p),
                det3x3(e, g, h, i, k, l, m, o, p),
                det3x3(e, f, h, i, j, l, m, n, p),
                det3x3(e, f, g, i, j, k, m, n, o),
            ],
            [
                det3x3(b0, c0, d0, j, k, l, n, o, p),
                det3x3(a0, c0, d0, i, k, l, m, o, p),
                det3x3(a0, b0, d0, i, j, l, m, n, p),
                det3x3(a0, b0, c0, i, j, k, m, n, o),
            ],
            [
                det3x3(b0, c0, d0, f, g, h, n, o, p),
                det3x3(a0, c0, d0, e, g, h, m, o, p),
                det3x3(a0, b0, d0, e, f, h, m, n, p),
                det3x3(a0, b0, c0, e, f, g, m, n, o),
            ],
            [
                det3x3(b0, c0, d0, f, g, h, j, k, l),
                det3x3(a0, c0, d0, e, g, h, i, k, l),
                det3x3(a0, b0, d0, e, f, h, i, j, l),
                det3x3(a0, b0, c0, e, f, g, i, j, k),
            ],
        ];

        // Cofactors
        let mat = [
            [a, -b, c, -d],
            [-e, f, -g, h],
            [i, -j, k, -l],
            [-m, n, -o, p],
        ];

        // Find determinant of original matrix based on the cofactors.
        let det = a0 * mat[0][0] + b0 * mat[0][1] + c0 * mat[0][2] + d0 * mat[0][3];

        // If determinant is zero, the inverse does not exist.
        if (-f32::EPSILON..=f32::EPSILON).contains(&det) {
            return None;
        }

        // "Divide" matrix by the determinant.
        let y = 1.0 / det;
        let mat: Vec<f32> = mat.iter().flatten().map(|x| x * y).collect();

        // Adjugate
        let mut mat = SquareMatrix4([
            mat[0..4].try_into().unwrap(),
            mat[4..8].try_into().unwrap(),
            mat[8..12].try_into().unwrap(),
            mat[12..16].try_into().unwrap(),
        ]);
        mat.transpose();

        Some(mat)
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
            [1.0, 0.0, 0.0, 0.0],
            [0.0, radians.cos(), -radians.sin(), 0.0],
            [0.0, radians.sin(), radians.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rot_y(radians: f32) -> Self {
        Self([
            [radians.cos(), 0.0, radians.sin(), 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-radians.sin(), 0.0, radians.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rot_z(radians: f32) -> Self {
        Self([
            [radians.cos(), -radians.sin(), 0.0, 0.0],
            [radians.sin(), radians.cos(), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}

impl std::ops::Mul for &SquareMatrix4 {
    type Output = SquareMatrix4;
    fn mul(self, rhs: &SquareMatrix4) -> Self::Output {
        // Left hand side
        let [[la, lb, lc, ld], [le, lf, lg, lh], [li, lj, lk, ll], [lm, ln, lo, lp]] = self.0;
        // Right hand side
        let [[ra, rb, rc, rd], [re, rf, rg, rh], [ri, rj, rk, rl], [rm, rn, ro, rp]] = rhs.0;

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

impl std::ops::Mul<&SquareMatrix4> for SquareMatrix4 {
    type Output = SquareMatrix4;
    fn mul(self, rhs: &SquareMatrix4) -> Self::Output {
        &self * rhs
    }
}

impl std::ops::Mul<&Vector4> for &SquareMatrix4 {
    type Output = Vector4;
    fn mul(self, v: &Vector4) -> Self::Output {
        let x = self.0[0][0] * v.x + self.0[0][1] * v.y + self.0[0][2] * v.z + self.0[0][3] * v.w;

        let y = self.0[1][0] * v.x + self.0[1][1] * v.y + self.0[1][2] * v.z + self.0[1][3] * v.w;

        let z = self.0[2][0] * v.x + self.0[2][1] * v.y + self.0[2][2] * v.z + self.0[2][3] * v.w;

        let w = self.0[3][0] * v.x + self.0[3][1] * v.y + self.0[3][2] * v.z + self.0[3][3] * v.w;

        Vector4 { x, y, z, w }
    }
}

impl std::fmt::Display for SquareMatrix4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:?}\n{:?}\n{:?}\n{:?}",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// NOTE Do not use this for anything other than debugging (and tests).
impl PartialEq for SquareMatrix4 {
    fn eq(&self, rhs: &Self) -> bool {
        let approx_equal = |lhs: f32, rhs: f32| -> bool {
            // NOTE This accuracy is selected based on the unit-test results!
            (lhs - rhs).abs() < 0.01
        };

        self.0
            .iter()
            .flatten()
            .zip(rhs.0.iter().flatten())
            .all(|(&l, &r)| approx_equal(l, r))
    }
}

#[cfg(test)]
mod tests {
    use super::SquareMatrix4 as M;

    static A: M = M([
        [1., 2., 3., 4.],
        [5., 6., 7., 8.],
        [9., 10., 11., 12.],
        [13., 14., 15., 16.],
    ]);

    static B: M = M([
        [0.4, 0., 2., 0.],
        [0., 0., 0.2, 1.],
        [0., 2., 1., 0.],
        [1., 0., 0., 0.],
    ]);

    static C: M = M([
        [1., 2., 3., 4.],
        [5., 6., 7., 8.],
        [9., 0., 1., 2.],
        [3., 4., 5., 6.],
    ]);

    #[test]
    fn test_identity() {
        assert_eq!(&M::identity() * &A, A,);
        assert_eq!(&M::identity() * &A * &M::identity(), A,);
        assert_eq!(&M::identity() * &M::identity(), M::identity())
    }

    #[test]
    fn test_transpose() {
        assert_eq!(
            A.transposed(),
            M([
                [1., 5., 9., 13.],
                [2., 6., 10., 14.],
                [3., 7., 11., 15.],
                [4., 8., 12., 16.],
            ])
        );

        let mut a_id = &A * &M::identity();
        assert_eq!(a_id, A);
        a_id.transpose();
        assert_eq!(
            a_id,
            M([
                [1., 5., 9., 13.],
                [2., 6., 10., 14.],
                [3., 7., 11., 15.],
                [4., 8., 12., 16.],
            ])
        );
    }

    #[test]
    fn test_determinant() {
        assert_eq!(B.determinant(), -4.);
        assert_eq!(C.determinant(), 0.);
    }

    #[test]
    fn test_inverse_none() {
        assert_eq!(C.inversed(), None);
    }

    #[test]
    fn test_inverse_some() {
        assert_eq!(
            B.inversed().unwrap(),
            M([
                [0.0000000, 0.0000000, 0.0000000, 1.0000000],
                [-0.25, 0.0000000, 0.5, 0.1],
                [0.5, 0.0000000, 0.0000000, -0.2],
                [-0.1, 1.0000000, 0.0000000, 0.0400000],
            ])
        );
    }

    #[test]
    fn test_some_combinations() {
        assert_eq!(
            &A * &B,
            M([
                [4.4, 6., 5.4, 2.],
                [10., 14., 18.2, 6.],
                [15.6, 22., 31., 10.],
                [21.2, 30., 43.8, 14.],
            ])
        );

        assert_eq!(
            &A * &B.inversed().unwrap()
                * &C
                * &M::identity().transposed()
                * &M::identity()
                * &A.transposed()
                * &B,
            M([
                [53164. / 25., 72072. / 25., 291648. / 125., 22468. / 25.],
                [132078. / 25., 179044. / 25., 723896. / 125., 55786. / 25.],
                [210992. / 25., 286016. / 25., 1156144. / 125., 89104. / 25.],
                [289906. / 25., 392988. / 25., 1588392. / 125., 122422. / 25.],
            ])
        );
    }
}
