use crate::raycast::{
    general::{
        Intersect,
        Intersection,
        Material
    },
    matrix,
    ray::Ray,
    vector::{Vector3, UnitVector3, Vector4},
};

/// Either a single 3D primitive (eg. sphere, plane) or a collection composed
/// of multiple primitives (eg. a pyramid made from triangles)
#[derive(serde::Deserialize, Clone)]
pub enum Object3D {
    Primitive(Primitive3D),
    Composite(Vec<Primitive3D>),
}

pub struct TransformableObject3D {
    transform: Option<matrix::SquareMatrix4>,
    object: Object3D,
}

// TODO This would require less memory (ie. not copying Object3D::Composites)
// if `object` was an Rc<Object3D>?
impl TransformableObject3D {
    pub fn new(
        mut transform: Option<matrix::SquareMatrix4>,
        object: Object3D
    ) -> Self {
        if transform.is_some() {
            // Inversed here in advance, because always used so
            transform = transform.map(|t| t.inversed());
        }
        Self { transform,  object }
    }
}

impl Intersect for TransformableObject3D {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        let choose_intersect = |object: &Object3D, r| match object {
            Object3D::Composite(xs) => {
                xs.iter()
                    .filter_map(|obj| obj.intersect(r, tmin))
                    // Select the intersection closest to ray
                    .reduce(|acc, x| if x.t < acc.t { x } else { acc })
            },
            Object3D::Primitive(x) => {
                x.intersect(r, tmin)
            }
        };

        if let Some(t) = &self.transform {
            let ray = Ray::with_transform(ray.origin, ray.direction, &t);
            choose_intersect(&self.object, &ray)
            // Transform the intersection normal to object space
            .map(|mut intr| {
                let normal_v4 = Vector4::from_v3(intr.normal.into(), 0.0);
                // TODO Is this transformation right? (see also ray.rs)
                intr.normal = (&t.transposed() * &normal_v4)
                    .xyz()
                    .normalized();

                intr
            })
        } else {
            choose_intersect(&self.object, ray)
        }
    }
}


#[derive(serde::Deserialize, Clone)]
pub enum Primitive3D {
    Sphere {
        origin: Vector3,
        radius: f32,
        material: Option<Material>,
    },
    Plane {
        offset: f32,
        normal: UnitVector3,
        material: Option<Material>,
    },
    Triangle {
        vertices: [Vector3;3],
        normal: UnitVector3,
        material: Option<Material>,
    },
}

impl Intersect for Primitive3D {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        match *self {
            Self::Sphere { origin, radius, material } =>
                sphere_intersect(ray, tmin, origin, radius, material),
            Self::Plane { offset, normal, material } =>
                plane_intersect(ray, tmin, offset, normal, material),
            Self::Triangle { vertices, normal, material } =>
                triangle_intersect(ray, tmin, vertices, normal, material),
        }
    }
}

fn sphere_intersect(
    ray: &Ray,
    tmin: f32,
    origin: Vector3,
    radius: f32,
    material: Option<Material>,
) -> Option<Intersection> {
    // Calculate the items for quadratic formula
    let to_ray_origin = ray.origin - origin;
    // NOTE `a` is just 1.0 as ray.direction should be normalized
    let (a, b, c) = (
        1.0,
        2.0 * Vector3::from(ray.direction).dot(&to_ray_origin),
        to_ray_origin.dot(&to_ray_origin) - radius.powi(2)
    );

    let discriminant = b.powi(2) - 4.0 * a * c;
    // Check that ray hits the sphere
    if discriminant < 0.0 { return None }

    // The distances from ray origin to intersection point
    let (t1, t2) = (
        (-b + discriminant.sqrt()) / (2.0 * a),
        (-b - discriminant.sqrt()) / (2.0 * a)
    );

    // Check that the intersection is greater than minimum and select the
    // intersection closest to ray origin
    // TODO Is this tmin float-comparison accurate enough?
    let closest =
        if      tmin < t1 && t1 < t2 { Some(t1) }
        else if tmin < t2 && t2 < t1 { Some(t2) }
        else { None };

    if let Some(t) = closest {
        let point = ray.cast(t);
        let normal = (point - origin).normalized();
        Some(
            Intersection {
                t,
                incoming: ray.direction,
                point,
                normal,
                material: material.unwrap_or_default()
            }
        )
    } else {
        None
    }
}

fn plane_intersect(
    ray: &Ray,
    tmin: f32,
    offset: f32,
    normal: UnitVector3,
    material: Option<Material>,
) -> Option<Intersection> {
    let denominator = ray.direction.dot(&normal);

    // This checks inequality to 0 in floating point
    if denominator < -f32::EPSILON || f32::EPSILON < denominator {
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
                    normal: normal,
                    material: material.unwrap_or_default(),
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
    ray: &Ray,
    tmin: f32,
    vertices: [Vector3;3],
    normal: UnitVector3,
    material: Option<Material>,
) -> Option<Intersection> {
    // Algorithm from:
    // https://courses.cs.washington.edu/courses/cse557/09au/lectures/extras/triangle_intersection.pdf

    // Line plane intersection:
    let normal_v: Vector3 = normal.into();
    // d = n * A, any vertex A will do as they are on the triangle plane
    let d = normal_v.dot(&vertices[0]);
    let denom = normal_v.dot(&ray.direction.into());

    // If ray and normal are orthogonal, then plane and ray are parallel
    if -f32::EPSILON <= denom && denom <= f32::EPSILON {
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
                material: material.unwrap_or_default(),
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
