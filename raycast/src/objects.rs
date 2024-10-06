use crate::{
    Intersect,
    Intersection,
    Material,
    matrix,
    ray::Ray,
    vector::{Vector3, UnitVector3, Vector4},
};

#[derive(Debug)]
pub struct Object3D {
    transform: Option<matrix::SquareMatrix4>,
    object: Vec<Shape>,
    material: Material,
}

// TODO This would require less memory (ie. not copying Object3D::Composites)
// if `object` was an Rc<Object3D>?
impl Object3D {
    pub fn new(
        transform: Option<matrix::SquareMatrix4>,
        object: Vec<Shape>,
        material: Option<Material>,
    ) -> Self {
        Self {
            // Inverse transform here in advance, because always used so
            transform: transform.map(|t| 
                t.inversed()
                    .unwrap_or_else(|| panic!("The matrix does not have an inverse: {}", t))
            ),
            object,
            material: material.unwrap_or_default(),
        }
    }
}

impl Intersect for Object3D {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        // Helper to reduce code duplication
        let get_intersection = |r| {
            self.object.iter()
                .filter_map(|obj| match *obj {
                    Shape::Sphere { origin, radius } => sphere_intersect(
                        origin, radius, r, tmin, self.material
                    ),
                    Shape::Plane { offset, normal } => plane_intersect(
                        offset, normal, r, tmin, self.material
                    ),
                    Shape::Triangle { vertices, normal } => triangle_intersect(
                        vertices, normal, r, tmin, self.material
                    ),
                    Shape::Torus { origin, inner_radius, tube_radius } => torus_intersect(
                        origin, inner_radius, tube_radius, r, tmin, self.material
                    ),
                })
                // Select the intersection closest to ray
                .reduce(|acc, x| if x.t < acc.t { x } else { acc })
        };

        if let Some(t) = &self.transform {
            let ray = Ray::with_transform(ray.origin, ray.direction, t);
            get_intersection(&ray)
                // If there was an intersection transform its normal to object
                // space 
                .map(|mut intr| {
                    let normal_v4 = Vector4::from_v3(intr.normal.into(), 0.0);
                    // TODO Is this transformation right? (see also ray.rs)
                    intr.normal = (&t.transposed() * &normal_v4)
                        .xyz()
                        .normalized();

                    intr
                })
        } else {
            get_intersection(ray)
        }
    }
}


#[derive(serde::Deserialize, Clone, Debug)]
pub enum Shape {
    Sphere {
        origin: Vector3,
        radius: f32,
    },
    Plane {
        offset: f32,
        normal: UnitVector3,
    },
    Triangle {
        vertices: [Vector3; 3],
        normal: UnitVector3,
    },
    Torus {
        origin: Vector3,
        inner_radius: f32,
        tube_radius: f32,
    },
}

fn sphere_intersect(
    origin: Vector3,
    radius: f32,
    ray: &Ray,
    tmin: f32,
    material: Material,
) -> Option<Intersection> {
    // Calculate the items for quadratic formula
    let to_ray_origin = ray.origin - origin;
    // NOTE `a` is just 1.0 as ray.direction should be normalized
    let (a, b, c) = (
        1.0,
        2.0 * Vector3::from(ray.direction).dot(&to_ray_origin),
        to_ray_origin.dot(&to_ray_origin) - radius.powi(2)
    );

    // Check that the intersection is greater than minimum and select the
    // intersection closest to ray origin
    let closest = solve_quadratic(a, b, c)
        .and_then(|xs| min_greater_than(tmin, &xs));

    if let Some(t) = closest {
        let point = ray.cast(t);
        let normal = (point - origin).normalized();
        Some(
            Intersection {
                t,
                incoming: ray.direction,
                point,
                normal,
                material,
            }
        )
    } else {
        None
    }
}

fn plane_intersect(
    offset: f32,
    normal: UnitVector3,
    ray: &Ray,
    tmin: f32,
    material: Material,
) -> Option<Intersection> {
    let denominator = ray.direction.dot(&normal);

    if !is_zero(denominator) {
        // Single point of intersection

        let nominator = {
            let v: Vector3 = normal.into();
            let c = offset + v.dot(&ray.origin);
            -c
        };
        let t = nominator / denominator;
        if tmin < t {
            return Some(
                Intersection {
                    t,
                    incoming: ray.direction,
                    point: ray.cast(t),
                    normal,
                    material,
                }
            )
        }
    }

    // Line is parallel to plane and if contained in it, the infinitely
    // thin plane will be invisible
    // (or more likely, the intersection is too close)
    None
}

fn triangle_intersect(
    vertices: [Vector3;3],
    normal: UnitVector3,
    ray: &Ray,
    tmin: f32,
    material: Material,
) -> Option<Intersection> {
    // Algorithm from:
    // https://courses.cs.washington.edu/courses/cse557/09au/lectures/extras/triangle_intersection.pdf

    // Line plane intersection:
    let normal_v: Vector3 = normal.into();
    // d = n * A, any vertex A will do as they are on the triangle plane
    let d = normal_v.dot(&vertices[0]);
    let denom = normal_v.dot(&ray.direction.into());

    // If ray and normal are orthogonal, then plane and ray are parallel
    if is_zero(denom) {
        return None
    }

    let t = (d - normal_v.dot(&ray.origin)) / denom;

    let q = ray.cast(t);

    // Check that q lies on triangle plane; "inside-outside" test
    let ba = vertices[1] - vertices[0];
    let cb = vertices[2] - vertices[1];
    let ac = vertices[0] - vertices[2];
    let qa = q - vertices[0];
    let qb = q - vertices[1];
    let qc = q - vertices[2];
    let x1 = Vector3::cross(&ba, &qa).dot(&normal_v);
    let x2 = Vector3::cross(&cb, &qb).dot(&normal_v);
    let x3 = Vector3::cross(&ac, &qc).dot(&normal_v);
    if tmin <= t && x1 >= 0.0 && x2 >= 0.0 && x3 >= 0.0 {
        return Some(
            Intersection {
                t,
                incoming: ray.direction,
                point: q,
                normal,
                material,
            }
        )
    }
    None

    // TODO
    //let a = SquareMatrix3::from([
    //    vertices[0] - vertices[1],
    //	vertices[0] - vertices[2],
    //	ray.direction.into(),
    //]).transposed();

    //let a_minus_ro = vertices[0] - ray.origin;

    //let beta_numerator = SquareMatrix3::from([
    //	a_minus_ro,
    //	a.col(1), // col(1)
    //	ray.direction.into(),
    //]).transposed();

    //let gamma_numerator = SquareMatrix3::from([
    //	a.col(0), // col(0)
    //	a_minus_ro,
    //	ray.direction.into(),
    //]).transposed();

    //let t_numerator = SquareMatrix3::from([
    //	a.col(0), // col(0)
    //	a.col(1), // col(1)
    //	a_minus_ro,
    //]).transposed();

    //// All of type f32
    //let a_determinant = a.determinant();
    //let beta = beta_numerator.determinant() / a_determinant;
    //let gamma = gamma_numerator.determinant() / a_determinant;
    //let t = t_numerator.determinant() / a_determinant;
    //let alpha = 1.0 - beta - gamma;

    //if 0.0 <= alpha && 0.0 <= beta && 0.0 <= gamma {
    //	let sum_of_baryms = alpha + beta + gamma;
    //	if 1.0 - f32::EPSILON <= sum_of_baryms
    //        && sum_of_baryms <= 1.0 + f32::EPSILON
    //        && tmin <= t {
    //        //let interpolated_normal =
    //        //    alpha * normals[0]
    //        //    + beta * normals[1]
    //        //    + gamma * normals[2];
    //        return Some(
    //                Intersection {
    //                t,
    //                point: ray.cast(t),
    //                normal: normal, //interpolated_normal,
    //                material: material,
    //            }
    //        )
    //    }
    //}
    //None
}

/// Kudos:
/// - http://cosinekitty.com/raytrace/chapter13_torus.html
/// - https://en.wikipedia.org/wiki/Quartic_equation
fn torus_intersect(
    origin: Vector3,
    ir: f32, // Inner radius.
    tr: f32, // Tube radius.
    ray: &Ray,
    tmin: f32,
    material: Material,
) -> Option<Intersection> {
    // Build the terms for torus equation.

    let ev = ray.direction;
    let dp = ray.origin;

    let g = 4.0 * ir.powi(2) * (ev.x().powi(2) + ev.y().powi(2));
    let h = 8.0 * ir.powi(2) * (dp.x * ev.x() + dp.y * ev.y());
    let i = 4.0 * ir.powi(2) * (dp.x.powi(2) + dp.y.powi(2));
    let j = ev.x().powi(2) + ev.y().powi(2) + ev.z().powi(2);
    let k = 2.0 * (dp.x * ev.x() + dp.y * ev.y() + dp.z * ev.z());
    let l = dp.x.powi(2) + dp.y.powi(2) + dp.z.powi(2) + ir.powi(2) - tr.powi(2);

    let (a, b, c, d, e) = (
        j.powi(2),
        2.0 * j * k,
        (2.0 * j * l + k.powi(2) - g),
        (2.0 * k * l - h),
        (l.powi(2) - i),
    );

    let closest = {
        let xs = solve_quartic(b, c, d, e);
        min_greater_than(tmin, &xs)
    };

    //if let Some(_) = closest {
    //    println!("{:?}", closest);
    //}

    if let Some(t) = closest {
        let point = ray.cast(t);
        let normal = {
            let p = (point - origin).normalized();
            let pshadow = Vector3{ x: p.x(), y: p.y(), z: 0.0 }.normalized();
            // Point in the center of tube.
            let q = ir * pshadow;
            (point - q).normalized()
        };
        Some(
            Intersection {
                t,
                incoming: ray.direction,
                point,
                normal,
                material,
            }
        )
    } else {
        None
    } 
}

fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<[f32; 2]> {
    let discriminant = b.powi(2) - 4.0 * a * c;
    // Check for hit at all.
    if !is_positive(discriminant) {
        None
    } else {
        Some([
            (-b + discriminant.sqrt()) / (2.0 * a),
            (-b - discriminant.sqrt()) / (2.0 * a)
        ])
    }
}

/// Solving:
/// a4 * x^4 + a3 * x^3 + a2 x^2 + a1 * x + a0 = 0
/// NOTE: a4 == 1.0
fn solve_quartic(a3: f32, a2: f32, a1: f32, a0: f32) -> Vec<f32> {
    let a4 = 1.0_f32;

    // Make the equation depressed.
    let a = (-3.0 * a3.powi(2)) / (8.0 * a4.powi(2))
        + a2 / a4;
    let b = a3.powi(3) / (8.0 * a4.powi(3))
        - (a3 * a2) / (2.0 * a4.powi(2))
        + a1 / a4;
    let c = (-3.0 * a3.powi(4)) / (256.0 * a4.powi(4))
        + (a2 * a3.powi(2)) / (16.0 * a4.powi(3))
        - (a3 * a1) / (4.0 * a4.powi(2))
        + a0 / a4;

    let mut ts = vec![];
    if is_zero(b) {
        // Solve biquadratic equation.
        if let Some([x1, x2]) = solve_quadratic(1.0, a, c) {
            // "Extract x" and also filter out any complex solutions.
            if is_positive(x1) {
                ts.push(x1.sqrt());
                ts.push(-x1.sqrt());
            }
            if is_positive(x2) {
                ts.push(x2.sqrt());
                ts.push(-x2.sqrt());
            } 
        }
    } else if let Some(ts_) = solve_depressed_quartic(a, b, c) {
        ts.extend(ts_);
    }
    // Wikipedia: "substituting ... x = u - B / 4A produces the values for x that solve the
    // original quartic"
    ts.into_iter().map(|x| x - (a3 / (4.0 * a4))).collect()
}

/// NOTE: Assuming b != 0.
fn solve_depressed_quartic(a: f32, b: f32, c: f32) -> Option<[f32; 4]> {
    let p = -(a.powi(2) / 12.0)
        - c;

    let q = -(a.powi(3) / 108.0)
        + (a * c) / 3.0
        - b.powi(2) / 8.0;

    let w_ = q.powi(2) / 4.0
        + p.powi(3) / 27.0;
    if !is_positive(w_) {
        return None;
    }
    let w__ = -(q / 2.0)
        + w_.sqrt();
    let w = w__.cbrt();

    let y = a / 6.0
        + w
        - p / (3.0 * w);

    let e_ = 2.0 * y - a;
    if !is_positive(e_) {
        return None;
    }
    let e = e_.sqrt();
    let f_ = (2.0 * b) / e;
    if !is_positive(f_) {
        return None;
    }
    let f = f_.sqrt();
    let g = -2.0 * y - a;

    Some([
        0.5 * (-e + (g + f)),
        0.5 * (-e - (g + f)),
        0.5 * ( e + (g - f)),
        0.5 * ( e - (g - f)),
    ])
}

fn is_positive(x: f32) -> bool {
    x > f32::EPSILON
}

/// Check inequality to 0 in floating point.
fn is_zero(x: f32) -> bool {
    (-f32::EPSILON..=f32::EPSILON).contains(&x)
}

fn cmp(lhs: &f32, rhs: &f32) -> std::cmp::Ordering {
    if is_zero(*lhs - *rhs) {
        std::cmp::Ordering::Equal
    } else if is_positive(*lhs - *rhs) {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Less
    }
}

fn min_greater_than(tmin: f32, ts: &[f32]) -> Option<f32> {
    // TODO Is this tmin float-comparison accurate enough?
    ts.iter()
        .min_by(|lhs, rhs| cmp(*lhs, *rhs))
        .and_then(|x| if tmin < *x { Some(*x) } else { None })
}

#[cfg(test)]
/// Kudos:
/// https://jwilson.coe.uga.edu/EMAT6680Fa09/Davenport/Solving%20Quartic%20Equations.pdf
/// https://mathmonks.com/quartic-function
/// https://www.onlinemath4all.com/roots-of-polynomial-of-degree-4.html
mod test_quartic {
    use super::*;

    fn assert_all_answers_found(answer: &[f32], result: &[f32]) {
        let mut answer = answer.to_vec();
        answer.sort_by(cmp);
        let mut result = result.to_vec();
        result.sort_by(cmp);
        for (a, r) in answer.iter().zip(result.iter()) {
            let x = a - r;
            if !is_zero(x) {
                assert_eq!(a, r);
            }
        }
    }

    #[test]
    fn four_real_roots_1_test() {
        let roots = [-13.82772, -3.37935, 2.72474, 8.48233];
        let ys = solve_quartic(
               6.0,
            -123.0,
            -126.0,
            1080.0,
        );
        assert_eq!(roots.len(), ys.len(), "Mismatch in amount of roots");
        assert_all_answers_found(&roots, &ys);
    }

    #[test]
    fn four_real_roots_2_test() {
        let roots = [-3.0, -2.0, 1.0, 2.0];
        let ys = solve_quartic(
             2.0,
            -7.0,
            -8.0,
            12.0,
        );
        assert_eq!(roots.len(), ys.len(), "Mismatch in amount of roots");
        assert_all_answers_found(&roots, &ys);
    }

    #[test]
    fn four_real_roots_3_test() {
        let roots = [2.0, 4.0, -3.0, -5.0];
        let ys = solve_quartic(
             2.0,
           -25.0,
           -26.0,
           120.0,
        );
        assert_eq!(roots.len(), ys.len(), "Mismatch in amount of roots");
        assert_all_answers_found(&roots, &ys);
    }

    #[test]
    fn two_real_roots_1_test() {
        let roots = [0.0, 1.19013];
        let ys = solve_quartic(
              5.0,
             17.0,
            -29.0,
             0.0,
        );
        assert_eq!(roots.len(), ys.len(), "Mismatch in amount of roots");
        assert_all_answers_found(&roots, &ys);
    }

    #[test]
    fn two_real_roots_2_test() {
        let roots = [-0.31082, 0.18978];
        let ys = solve_quartic(
              1.0,
             17.0,
              2.0,
             -1.0,
        );
        assert_eq!(roots.len(), ys.len(), "Mismatch in amount of roots");
        assert_all_answers_found(&roots, &ys);
    }
}
