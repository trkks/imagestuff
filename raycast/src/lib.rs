pub mod color;
pub mod math;
pub mod ray;
pub use color::*;
use math::vector::Vector;
pub use math::*;

fn hit_sphere(center: vector::Vector3, radius: f32, ray: &ray::Ray) -> bool {
    let oc = ray.origin - center;
    let a = ray.direction.length_squared();
    let half_b = oc.dot(&ray.direction);
    let c = oc.length_squared() - radius * radius;
    let discriminant = half_b * half_b - a * c;
    discriminant > 0.0
}

pub fn shade(ray: ray::Ray) -> color::Color {
    if hit_sphere(
        vector::Vector3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        },
        0.5,
        &ray,
    ) {
        return color::Color::new(1.0, 0.0, 0.0);
    }

    let a = 0.5 * (ray.direction.y() + 1.0);
    color::Color::new(1.0, 1.0, 1.0) * (1.0 - a) + color::Color::new(0.5, 0.7, 1.0) * a
}
