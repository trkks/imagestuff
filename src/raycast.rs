use std::env::{Args};
use image::{ImageBuffer, Rgb};

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
impl std::ops::Mul<f32> for Vector3 {
    type Output = Self;
    fn mul(self, c: f32) -> Self::Output {
        Vector3::new(self.x * c, self.y * c, self.z * c)
    }
}
impl std::ops::Add for Vector3 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Vector3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}
impl std::ops::Sub for Vector3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Vector3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}
impl std::ops::Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}
impl From<Vector3> for Rgb<u16> {
    fn from(v: Vector3) -> Self { 
        Rgb(
            [ (v.x.clamp(0.0, 1.0) * (u16::MAX as f32)) as u16,
              (v.y.clamp(0.0, 1.0) * (u16::MAX as f32)) as u16,
              (v.z.clamp(0.0, 1.0) * (u16::MAX as f32)) as u16 ]
        )
    }
}

type SphereIntersection = Option<(f32, Vector3)>;
type ZBounds = (f32, f32); // Represents the depth-bounds of the view-frustum

struct Sphere {
    pub origin: Vector3,
    pub radius: f32,
}
impl Sphere {
    pub fn intersect(&self, ray: &Ray, zbs: ZBounds) -> SphereIntersection {
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
        // select the normal of intersection point closest to ray origin
        if        zbs.0 <= tt.0 && tt.0 <= zbs.1 && tt.0 < tt.1 {
            Some( (tt.0, (ray.direction() - self.origin).normalized()) )
        } else if zbs.0 <= tt.1 && tt.1 <= zbs.1 {
            Some( (tt.1, (ray.direction() - self.origin).normalized()) )
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
    pub fn cast(&self, t: f32) -> Vector3 {
        self.origin + (self.direction * t)
    }
}

pub fn run(mut args: Args) -> Result<(), String> {
    // Image bounds
    let (width, height) = match (args.next(), args.next()) {
        (Some(a),None   ) => (a.parse().unwrap(),a.parse().unwrap()),
        (Some(a),Some(b)) => (a.parse().unwrap(),b.parse().unwrap()),
        _                 => (1024, 1024),
    };

    // Depth-bounds of the frustum
    let zbs: ZBounds = (-1.0, 2.0);
    // Camera fov is 90 degrees ("fov" is named "angle" for reasons(?))
    let angle = std::f32::consts::PI / 2.0;
    let focal_point = Vector3::new(0.0,0.0,-2.0);
    // Sphere at origin
    let sphere = Sphere { origin: Vector3::new(0.0,0.0,0.0), radius: 1.0 };

    // Color used:
    let sphere_diffuse      = Vector3::new(0.0,0.6,0.8); // light blue?
    let sphere_specular     = Vector3::new(1.0,0.0,0.0); // red
    let background_color    = Vector3::new(0.0,0.0,0.0); // black

    // Render:
    let image = ImgBuffer16::from_fn(width as u32, height as u32, |ix, iy| {

        // Calculate image plane coordinates x,y (so that they're in [-1, 1])
        let y: f32 = (iy as f32 / height as f32) * 2.0 - 1.0;  
        let x: f32 = (ix as f32 / width  as f32) * 2.0 - 1.0;  

        // TODO Google this z-formula; smth to do with fov and trigonometry
        let z = 1.0 / f32::tan(angle / 2.0);

        // Generate ray from camera to the image plane.
        // NOTE this is not a general form, as the camera now has hardcoded
        // horizontal = (1,0,0), up = (0,1,0), direction =  (0,0,1).
        let direction = Vector3::new(x, y, z).normalized();

        let ray = Ray::new(focal_point, direction);
        
        let color = if let Some((t, normal)) = sphere.intersect(&ray, zbs) {
            // Shade the pixel on sphere (light position == (0,1,-.5)):

            let surface_to_light = Vector3::new(0.0,1.0,-0.5) - ray.cast(t);

            // Diffuse shading
            let diffuse_amount = surface_to_light.dot(normal).clamp(0.0, 1.0);

            // Specular shading:
            let v = normal * (-ray.direction()).dot(normal) * 2.0
                    + ray.direction();
            let specular_amount = surface_to_light.dot(v)
                                    .clamp(0.0, 1.0)
                                    .powf(1.0); // to the power of shininess


            background_color 
            + (sphere_diffuse * diffuse_amount)
            + (sphere_specular * specular_amount)

        } else {
            // Shade the pixel on background
            background_color
        };

        color.into()

    });

    // Write to image file
    image.save("raycast_sphere.png").unwrap(); // TODO Handle error-result

    Ok(())
}
