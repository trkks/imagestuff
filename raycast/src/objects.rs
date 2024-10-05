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

    // This checks inequality to 0 in floating point
    if !(-f32::EPSILON..=f32::EPSILON).contains(&denominator) {
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
    if (-f32::EPSILON..=f32::EPSILON).contains(&denom) {
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
    hr: f32, // Hole radius.
    tr: f32, // Tube radius.
    ray: &Ray,
    tmin: f32,
    material: Material,
) -> Option<Intersection> {
    let ev = ray.direction;
    let dp = ray.origin;

    let g = 4.0 * hr.powi(2) * (ev.x().powi(2) + ev.y().powi(2));
    let h = 8.0 * hr.powi(2) * (dp.x * ev.x() + dp.y * ev.y());
    let i = 4.0 * hr.powi(2) * (dp.x.powi(2) + dp.y.powi(2));
    let j = ev.x().powi(2) + ev.y().powi(2) + ev.z().powi(2);
    let k = 2.0 * (dp.x * ev.x() + dp.y * ev.y() + dp.z * ev.z());
    let l = dp.x.powi(2) + dp.y.powi(2) + dp.z.powi(2) + hr.powi(2) - tr.powi(2);

    let (a_, b_, c_, d, e) = (
        j.powi(2),
        2.0 * j * k,
        (2.0 * j * l + k.powi(2) - g),
        (2.0 * k * l - h),
        (l.powi(2) -i),
    );

    // TODO "If b is not already zero..."
    
    // Make the equation depressed.
    let a = (-3.0 * b_.powi(2)) / (8.0 * a_.powi(2))
        + c_ / a_;
    let c = (-3.0 * b_.powi(4)) / (256.0 * a_.powi(4))
        + (c_ * b_.powi(2)) / (16.0 * a_.powi(3))
        - (b_ * d) / (4.0 * a_.powi(2))
        + e / a_;

    let closest = if (-f32::EPSILON..=f32::EPSILON).contains(&b_) {
        // Solve biquadratic equation.
        solve_quadratic(1.0, a, c)
            .map(|xs|{
                // Filter out complex solutions.
                let t1 = if xs[0] < 0.0 { tmin } else {  xs[0].sqrt() };
                let t2 = if xs[0] < 0.0 { tmin } else { -xs[0].sqrt() };
                let t3 = if xs[1] < 0.0 { tmin } else {  xs[1].sqrt() };
                let t4 = if xs[1] < 0.0 { tmin } else { -xs[1].sqrt() };
                return [t1, t2, t3, t4];
            })
    } else {
        let b = (b_.powi(3) / 8.0 * a_.powi(3))
            - (b_ * c_) / (2.0 * a_.powi(2))
            + d / a_;
 
        solve_depressed_quartic(a, b, c)
    }.and_then(|xs| min_greater_than(tmin, &xs));
    if let Some(_) = closest {
        println!("{:?}", closest);
    }

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

fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<[f32; 2]> {
    let discriminant = b.powi(2) - 4.0 * a * c;
    // Check for hit at all.
    if discriminant < 0.0 {
        None
    } else {
        Some([
            (-b + discriminant.sqrt()) / (2.0 * a),
            (-b - discriminant.sqrt()) / (2.0 * a)
        ])
    }
}

/// NOTE: Assuming b != 0.
fn solve_depressed_quartic(a: f32, b: f32, c: f32) -> Option<[f32; 4]> {
    let p = -(a.powi(2) / 12.0)
        - c;
    let q = -(a.powi(3) / 108.0)
        + (a * c) / 3.0
        - b.powi(2) / 8.0;
    let w = (
        -(q / 2.0)
        + (
            q.powi(2) / 4.0
            + p.powi(3) / 27.0
        ).sqrt()
    ).cbrt();
    let y = a / 6.0
        + w
        - p / (3.0 * w);
    Some([
        0.5 * (-(2.0 * y - a).sqrt() + (-2.0 * y - a + ((2.0 * b) / (2.0 * y - a).sqrt()).sqrt())),
        0.5 * (-(2.0 * y - a).sqrt() - (-2.0 * y - a + ((2.0 * b) / (2.0 * y - a).sqrt()).sqrt())),
        0.5 * ( (2.0 * y - a).sqrt() + (-2.0 * y - a - ((2.0 * b) / (2.0 * y - a).sqrt()).sqrt())),
        0.5 * ( (2.0 * y - a).sqrt() - (-2.0 * y - a - ((2.0 * b) / (2.0 * y - a).sqrt()).sqrt())),
    ])
}

fn min_greater_than(tmin: f32, ts: &[f32]) -> Option<f32> {
    // TODO Is this tmin float-comparison accurate enough?
    ts.iter()
        .min_by(|lhs, rhs|
            if lhs < rhs {
                std::cmp::Ordering::Less
            } else if lhs > rhs {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        ).and_then(|x| if tmin < *x { Some(*x) } else { None })
}
