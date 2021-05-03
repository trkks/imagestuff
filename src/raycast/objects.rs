use crate::raycast::{
    general::{Material, Intersect, Intersection, Ray, color},
    vector3::Vector3,
};

pub struct Sphere {
    origin: Vector3,
    radius: f32,
    material: Material,
}
impl Sphere {
    pub fn new(origin: Vector3, radius: f32) -> Self {
        Sphere { 
            origin, 
            radius, 
            material: Material { color: color::consts::RED }
        }
    }
}
impl Intersect for Sphere {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        // Calculate the items for quadratic formula
        let to_ray_origin = ray.origin() - self.origin;
        // NOTE `a` could be just 1.0 if ray.direction is normalized
        let (a, b, c) = (
            ray.direction().dot(ray.direction()),
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
        let opt = if tmin <= t1 && t1 < t2 { Some(t1) } 
            else if  tmin <= t2 && t2 < t1 { Some(t2) } 
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

/// A collection of objects
pub struct Scene {
    _ambient_color: Vector3,
    _lights: Vec<Vector3>,
    spheres: Vec<Sphere>,
}
impl Scene {
    pub fn intersect(&self, ray: &Ray, tmin: f32) 
        -> Option<Intersection>
    {
        self.spheres.iter()
            // Intersect the objects
            .filter_map(|sphere| sphere.intersect(&ray, tmin))
            // Select the intersection closest to ray
            .reduce(|acc, x| if x.t < acc.t { x } else { acc })
    }
    pub fn from(spheres: Vec<Sphere>) -> Self {
        Scene {
            _ambient_color: Vector3::new(0.0, 0.0, 0.0),
            _lights: vec![],
            spheres,
        }
    }
}
