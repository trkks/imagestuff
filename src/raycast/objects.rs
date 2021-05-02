use crate::raycast::vector3::Vector3;
use crate::raycast::ZBounds;

type SphereIntersection = Option<(f32, Vector3)>;

pub struct Sphere {
    origin: Vector3,
    radius: f32,
}
impl Sphere {
    pub fn new(origin: Vector3, radius: f32) -> Self {
        Sphere { origin, radius }
    }

    pub fn intersect(&self, ray: &Ray, zbs: ZBounds) -> SphereIntersection {
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

        // Check that the intersections are inside the depth-bounds and 
        // select the normal of intersection point closest to ray origin
        if        zbs.0 <= t1 && t1 <= zbs.1 && t1 < t2 {
            Some( (t1, (ray.direction() - self.origin).normalized()) )
        } else if zbs.0 <= t2 && t2 <= zbs.1 && t2 < t1 {
            Some( (t2, (ray.direction() - self.origin).normalized()) )
        } else {
            None
        }
    }
}

pub struct Ray {
    origin: Vector3,
    direction: Vector3,
}
impl Ray {
    pub fn new(origin: Vector3, direction: Vector3) -> Self {
        // TODO Normalize direction?
        Ray { origin, direction: direction }   
    }
    pub fn origin(&self) -> Vector3 { self.origin }
    pub fn direction(&self) -> Vector3 { self.direction }
    pub fn cast(&self, t: f32) -> Vector3 {
        self.origin + (self.direction * t)
    }
}
