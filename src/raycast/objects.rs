/// This module contains objects that are `Intersect`

use std::convert::TryFrom;

use crate::raycast::{
    general::{color, Intersect, Intersection, Light, Material},
    ray::Ray,
    vector3::{Vector3, UnitVector3},
    matrix::SquareMatrix3,
};

/// A collection of objects
pub struct Scene {
    pub ambient_color: color::Color,
    pub lights: Vec<Light>,
    spheres: Vec<Sphere>,
    planes: Vec<Plane>,
    triangles: Vec<Triangle>,
}
impl TryFrom<serde_json::Value> for Scene {
    type Error = serde_json::Error;

    fn try_from(
        mut json: serde_json::Value,
    ) -> Result<Self,serde_json::Error> {
        let ambient_color =
            serde_json::from_value(json["ambient_color"].take())?;
        let lights = serde_json::from_value(json["lights"].take())?;
        let spheres = serde_json::from_value(json["spheres"].take())?;
        // TODO The planes' normals are not deserialized into unit vectors
        let planes = serde_json::from_value(json["planes"].take())?;
        let triangles = serde_json::from_value::<Vec<serde_json::Value>>(json["triangles"].take())?
            .iter_mut()
            .map(|x| {
                let vertices: [Vector3;3] =
                    serde_json::from_value(x["vertices"].take()).unwrap();
                // NOTE Order of vertices is relevant for the normal
                // Here the right hand rule is used (counter clockwise order)
                let u = vertices[1] - vertices[0];
                let v = vertices[2] - vertices[0];
                let normal = Vector3::cross(&u, &v).normalized();
                let material =
                    serde_json::from_value(x["material"].take()).unwrap();

                Triangle { vertices, normal, material }
            })
            .collect();

        Ok(Scene { ambient_color, lights, spheres, planes, triangles })
    }
}

impl Intersect for Scene {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        let spheres = self.spheres.iter()
            .filter_map(|obj| obj.intersect(ray, tmin));
        let planes = self.planes.iter()
            .filter_map(|obj| obj.intersect(ray, tmin));
        let triangles = self.triangles.iter()
            .filter_map(|obj| obj.intersect(&ray, tmin));

        spheres
            .chain(planes)
            .chain(triangles)
            // Select the intersection closest to ray
            .reduce(|acc, x| if x.t < acc.t { x } else { acc })
    }
}

#[derive(serde::Deserialize)]
pub struct Sphere {
    origin: Vector3,
    radius: f32,
    material: Material,
}
impl Intersect for Sphere {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        // Calculate the items for quadratic formula
        let to_ray_origin = ray.origin - self.origin;
        // NOTE `a` is just 1.0 as ray.direction should be normalized
        let (a, b, c) = (
            1.0,
            2.0 * Vector3::from(ray.direction).dot(&to_ray_origin),
            to_ray_origin.dot(&to_ray_origin) - self.radius.powi(2)
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
        let opt = if tmin < t1 && t1 < t2 { Some(t1) } 
            else if  tmin < t2 && t2 < t1 { Some(t2) } 
            else { None };

        if let Some(t) = opt {
            let point = ray.cast(t);
            let normal = (point - self.origin).normalized();
            Some(Intersection {t, point, normal, material: self.material })
        } else {
            None
        }
    }
}

#[derive(serde::Deserialize)]
pub struct Plane {
    offset: f32,
    normal: UnitVector3,
    material: Material,
}
impl Intersect for Plane {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        let denominator = ray.direction.dot(&self.normal);

        // This checks inequality to 0 in floating point
        if denominator < -f32::EPSILON || f32::EPSILON < denominator {
            // Single point of intersection

            let nominator = {
                let v: Vector3 = self.normal.into();
                let c = self.offset + v.dot(&ray.origin);
                -c
            };
            let t = nominator / denominator;
            if tmin < t {
                return Some(
                    Intersection {
                        t,
                        point: ray.cast(t),
                        normal: self.normal,
                        material: self.material,
                    }
                )
            }
        }

        // Line is parallel to plane and if contained in it, the infinitely
        // thin plane will be invisible
        // (or more likely, the intersection is too close)
        None
    }
}

#[derive(serde::Deserialize)]
pub struct Triangle {
    vertices: [Vector3;3],
    normal: UnitVector3,
    material: Material,
}
impl Intersect for Triangle {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        // Algorithm from:
        // https://courses.cs.washington.edu/courses/cse557/09au/lectures/extras/triangle_intersection.pdf

        // Line plane intersection:
        let normal: Vector3 = self.normal.into();
        // d = n * A, any vertex A will do as they are on the triangle plane
        let d = normal.dot(&self.vertices[0]);
        let denom = normal.dot(&ray.direction.into());

        // If ray and normal are orthogonal, then plane and ray are parallel
        if -f32::EPSILON <= denom && denom <= f32::EPSILON {
            return None
        }

        let t = (d - normal.dot(&ray.origin)) / denom;

        let q = ray.cast(t);

        // Check that q lies on triangle plane; "inside-outside" test
        let ba = self.vertices[1] - self.vertices[0];
        let cb = self.vertices[2] - self.vertices[1];
        let ac = self.vertices[0] - self.vertices[2];
        let qa = q - self.vertices[0];
        let qb = q - self.vertices[1];
        let qc = q - self.vertices[2];
        let x1 = Vector3::cross(&ba, &qa).dot(&normal);
        let x2 = Vector3::cross(&cb, &qb).dot(&normal);
        let x3 = Vector3::cross(&ac, &qc).dot(&normal);
        if tmin <= t && x1 >= 0.0 && x2 >= 0.0 && x3 >= 0.0 {
            return Some(
                Intersection {
                    t,
                    point: q,
                    normal: self.normal,
                    material: self.material,
                }
            )
        }
        None

        // TODO
        //let a = SquareMatrix3::from([
        //    self.vertices[0] - self.vertices[1],
        //	self.vertices[0] - self.vertices[2],
        //	ray.direction.into(),
        //]).transposed();

        //let a_minus_ro = self.vertices[0] - ray.origin;

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
        //        //    alpha * self.normals[0]
        //        //    + beta * self.normals[1]
        //        //    + gamma * self.normals[2];
        //        return Some(
        //                Intersection {
        //                t,
        //                point: ray.cast(t),
        //                normal: self.normal, //interpolated_normal,
        //                material: self.material,
        //            }
        //        )
        //    }
        //}
        //None
    }
}
