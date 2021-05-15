use crate::raycast::{
    general::{Material, Intersect, Intersection, Ray, color},
    vector3::Vector3,
};
use serde;

/// A collection of objects
#[derive(serde::Deserialize)]
pub struct Scene {
    ambient_color: color::Color,
    lights: Vec<Light>,
    spheres: Vec<Sphere>,
    planes: Vec<Plane>,
}
impl Scene {
    pub fn intersect(&self, ray: &Ray, tmin: f32)
        -> Option<Intersection>
    {
        self.spheres.iter()
            // Intersect the spheres
            .filter_map(|obj| obj.intersect(&ray, tmin))
            .chain(
                self.planes.iter()
                // Intersect the planes
                .filter_map(|obj| obj.intersect(&ray, tmin))
            )
            // Select the intersection closest to ray
            .reduce(|acc, x| if x.t < acc.t { x } else { acc })
    }
    #[allow(dead_code)]
    pub fn new(ambient_color: color::Color,
               spheres: Vec<Sphere>, 
               planes: Vec<Plane>, 
               lights: Vec<Light>) -> Self {
        Scene {
            ambient_color,
            lights,
            spheres,
            planes,
        }
    }
    pub fn lights(&self) -> &Vec<Light> {
        &self.lights
    }
    pub fn ambient_color(&self) -> color::Color {
        self.ambient_color
    }
}

#[derive(serde::Deserialize)]
pub struct Sphere {
    origin: Vector3,
    radius: f32,
    material: Material,
}
impl Sphere {
    #[allow(dead_code)]
    pub fn new(origin: Vector3, radius: f32, material: Material) -> Self {
        Sphere { 
            origin,
            radius, 
            material,
        }
    }
}
impl Intersect for Sphere {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        // Calculate the items for quadratic formula
        let to_ray_origin = ray.origin() - self.origin;
        // NOTE `a` is just 1.0 as ray.direction should be normalized
        let (a, b, c) = (
            1.0,
            2.0 * ray.direction().dot(to_ray_origin),
            to_ray_origin.dot(to_ray_origin) - self.radius.powi(2)
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
            Some( Intersection::new(t, point, normal, self.material) )
        } else {
            None
        }
    }
}

#[derive(serde::Deserialize)]
pub struct Plane {
    origin: Vector3,
    normal: Vector3,
    material: Material,
}
impl Intersect for Plane {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        let denominator = ray.direction().dot(self.normal);

        // This checks inequality to 0 in floating point
        if denominator < -f32::EPSILON || f32::EPSILON < denominator {
            // Single point of intersection

            let d = -(self.normal.dot(self.origin));
            let nominator = -(d + self.normal.dot(ray.origin()));
            let t = nominator / denominator;
            if tmin < t {
                return Some(
                    Intersection::new(
                        t,
                        ray.cast(t),
                        self.normal,
                        self.material
                    )
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
pub struct Light {
    pub position: Vector3,
    pub _direction: Option<Vector3>,
    pub color: color::Color,
    pub intensity: f32,
}
impl Light {
    /// Scales the light's intensity relative to a target's position
    //TODO This might very well be stupid
    pub fn color_to_target(&self, target_pos: Vector3) -> color::Color {
        let to_target = target_pos - self.position;
        // Normalize to lights intensity or in other words "range"
        let mut factor = self.intensity / to_target.length(); // NOTE x / 0
        if let Some(_dir) = self._direction {
            // Scale with relation to direction's "cone" (eg. dead-on => max)
            //TODO if angle between direction and to_target is small => max
            factor *= 1.0;
        }

        self.color * factor
    }
}
