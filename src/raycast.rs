use image::{ImageBuffer, Rgb};
use std::convert::TryInto;

type ImgBuffer16 = ImageBuffer::<Rgb<u16>, Vec<u16>>;

#[derive(Copy,Clone,Debug)]
struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }
    pub fn normalized(&self) -> Self {
        let length = self.length();
        Vector3::new(self.x / length, self.y / length, self.z / length)
    }
    pub fn length(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }
    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}
impl std::ops::Sub for Vector3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Vector3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

type SphereIntersection = Option<f32>;
type ZBounds = (f32, f32); // Represents the depth-bounds of the view-frustum

struct Sphere {
    pub origin: Vector3,
    pub radius: f32,
}
impl Sphere {
    pub fn intersect(&self, ray: Ray, zbs: ZBounds) -> SphereIntersection {
        // Calculate the items for quadratic formula
        let to_ray_origin = ray.origin() - self.origin;
        let (a, b, c) = (
            ray.direction().dot(ray.direction()),
            2.0 * ray.direction().dot(to_ray_origin),
            to_ray_origin.dot(to_ray_origin) - self.radius.powi(2)
        );

        let discriminant = b * b - 4.0 * a * c;
        // Check that ray hits the sphere
        if discriminant < 0.0 { return None }
                    
        // The distances from ray origin to intersection point
        let tt = (
            (-b + discriminant.sqrt()) / (2.0 * a),
            (-b - discriminant.sqrt()) / (2.0 * a)
        );

        // Check that the intersections are inside the depth-bounds and 
        // select the intersection closest to ray origin
        if        zbs.0 <= tt.0 && tt.0 <= zbs.1 && tt.0 < tt.1 {
            Some(tt.0)
        } else if zbs.0 <= tt.1 && tt.1 <= zbs.1 {
            Some(tt.1)
        } else {
            None
        }
    }
}

struct Ray {
    origin: Vector3,
    direction: Vector3,
}
impl Ray {
    pub fn new(origin: Vector3, direction: Vector3) -> Self {
        // TODO Normalize direction?
        Ray { origin, direction }   
    }
    pub fn origin(&self) -> Vector3 { self.origin }
    pub fn direction(&self) -> Vector3 { self.direction }
}

pub fn run() -> Result<(), String> {
    // Depth-bounds of the frustum
    let zbs: ZBounds = (-1.0, 2.0);
    // Image of 256x256 pixels
    let (width, height) = (64, 64);
    let mut image: Vec<[u16;3]> = Vec::with_capacity(width * height);
    // Camera fov is 90 degrees ("fov" is named "angle" for reasons(?))
    let angle = std::f32::consts::PI / 2.0;
    let focal_point = Vector3::new(0.0,0.0,-2.0);
    // Sphere at origin
    let sphere = Sphere { origin: Vector3::new(0.0,0.0,0.0), radius: 1.0 };
    // Color of the sphere (Currently: white)
    let color = Rgb([0xffff,0xffff,0xffff]);

    // Render:
    for iy in 0..height {
        // Calculate image plane coordinates x,y (so that they're in [-1, 1])
        let y: f32 = (iy as f32 / height as f32) * 2.0 - 1.0;  
        for ix in 0..width {
            // Same operation as for `y` above
            let x: f32 = (ix as f32 / width as f32) * 2.0 - 1.0;  

            // TODO Google this z-formula; smth to do with fov and trigonometry
            let z = 1.0 / f32::tan(angle / 2.0);

            // Generate ray from camera to the image plane.
            let direction = Vector3::new(x, y, z).normalized();
            let ray = Ray::new(focal_point, direction);
            
            if let Some(t) = sphere.intersect(ray, zbs) {
                println!("intensity: {}", t);
                // Set the pixel to correct color intensity on sphere 
                image.push( 
                    color.0.iter().map(|c| (*c as f32 * (t / zbs.1)) as u16)
                                  .collect::<Vec<u16>>()
                                  .try_into()
                                  .unwrap() // Can't (or shouldn't?) panic
                );
            } else {
                // Background is black
                image.push([0,0,0]);
            }
        }
    }

    // Write to image file
    ImgBuffer16::from_vec(width as u32, height as u32, image.concat())
        .unwrap()
        .save("raycast_sphere.png")
        .unwrap(); // TODO Handle error-result

    Ok(())
}
