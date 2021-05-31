/// This module contains objects that are `Intersect`

use crate::raycast::{
    general::{color, Intersect, Intersection, Light, Material},
    ray::Ray,
    vector3::{Vector3, UnitVector3},
};

/// A collection of objects
pub struct Scene {
    pub ambient_color: color::Color,
    pub lights: Vec<Light>,
    spheres: Vec<Sphere>,
    planes: Vec<Plane>,
}
impl Scene {
    pub fn from_json(
        mut json: serde_json::Value,
    ) -> Result<Self,serde_json::Error> {
        let ambient_color =
            serde_json::from_value(json["ambient_color"].take())?;
        let lights = serde_json::from_value(json["lights"].take())?;
        let spheres = serde_json::from_value(json["spheres"].take())?;
        // NOTE/TODO The planes' normals are not deserialized into unit vectors
        let planes = serde_json::from_value(json["planes"].take())?;

        Ok(Scene { ambient_color, lights, spheres, planes })
    }
}

impl Intersect for Scene {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        let spheres = self.spheres.iter()
            .filter_map(|obj| obj.intersect(ray, tmin));
        let planes = self.planes.iter()
            .filter_map(|obj| obj.intersect(ray, tmin));
        // TODO?
        //let triangles = self.triangles.iter()
        //    .filter_map(|obj| obj.intersect(&ray, tmin));

        spheres.chain(planes)//.chain(triangles)
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
