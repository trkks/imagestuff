mod vector3; // is this called a re-export?
mod objects; // is this called a re-export?

use crate::raycast::vector3::Vector3;
use crate::raycast::objects::{Sphere, Ray};

use std::env::{Args};
use image::{ImageBuffer, Rgb};

type ImgBuffer16 = ImageBuffer::<Rgb<u16>, Vec<u16>>;

pub fn run(mut args: Args) -> Result<(), String> {
    // Image bounds
    let (width, height) = match (args.next(), args.next()) {
        (Some(a),None   ) => (a.parse().unwrap(),a.parse().unwrap()),
        (Some(a),Some(b)) => (a.parse().unwrap(),b.parse().unwrap()),
        _                 => (1024, 1024),
    };

    // Camera fov is 90 degrees
    let camera = PerspectiveCamera::new(
        Vector3::new(0.0, 0.0,-2.0), // position
        Vector3::new(0.0, 0.0, 1.0), // direction
        Vector3::new(0.0, 1.0, 0.0), // up
        std::f32::consts::PI / 2.0,  // fov
        (0.0, 4.0)                   // near and far bounds of view
    );

    // Spheres in scene
    let sphere  = Sphere::new(Vector3::new(0.0,0.0,0.0), 1.0);
    //let sphere2 = Sphere::new(Vector3::new(-0.2,0.5,0.0), 0.8);

    let shininess = 1.0;
    // A single light source in the scene
    let light_position = Vector3::new(0.0, 1.0, -1.0);

    // Colors used:
    let sphere_diffuse      = Vector3::new(0.0, 0.6, 0.8); // light blue?
    let sphere_specular     = Vector3::new(1.0, 0.0, 0.0); // red
    let background_color    = Vector3::new(0.0, 0.0, 0.0); // black

    // Render:
    let image = ImgBuffer16::from_fn(width as u32, height as u32, |ix, iy| {

        // Calculate image plane coordinates x,y so that they're in [-1, 1]
        let x: f32 = (ix as f32 / width  as f32) * 2.0 - 1.0;  
        let y: f32 = (iy as f32 / height as f32) * 2.0 - 1.0;  
        let ray = camera.shoot_at(x, y);

        let shade = |t, normal| {
            // Run the raytracing: 
            // TODO Feed the scene to a function for intersecting recursively
            // for n iterations or until intersection returns None. Ray
            // direction is the reflection and ray origin is the intersection
            // point
            //let iterations = 2;
            //let mut color = background_color;

            // Shade the pixel on sphere (light position == (0,1,-.5)):
            let surface_to_light = (light_position - ray.cast(t))
                                   .normalized();

            // Diffuse shading:
            let diffuse_amount = surface_to_light.dot(normal).clamp(0.0, 1.0);

            // Specular shading: 
            let reflection = (normal * surface_to_light.dot(normal) * 2.0
                             - surface_to_light)
                             .normalized();
            let specular_amount = reflection.dot(-ray.direction())
                                    .clamp(0.0, 1.0)
                                    .powf(shininess);

            background_color 
            + (sphere_diffuse * diffuse_amount)
            + (sphere_specular * specular_amount)
        };
        
        let mut the_intersects = vec![];
        if let Some(intersection) = sphere.intersect(&ray, camera.bounds()) {
            the_intersects.push(intersection);
        }
//        if let Some(intersection) = sphere2.intersect(&ray, zbs) {
//            the_intersects.push(intersection);
//        }

        // Select the closest intersection for rendering
        let closest = the_intersects.iter().fold((None, f32::MAX), 
            |(opt,min_t), elem| 
            if elem.0 < min_t { (Some(elem), elem.0) } else { (opt, min_t) }
        ).0;

        let color = if let Some(&(t, normal)) = closest {
            shade(t, normal)
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

struct PerspectiveCamera {
    position: Vector3,
    direction: Vector3,
    horizontal: Vector3,
    up: Vector3,
    fov: f32,
    view_bounds: (f32, f32), // Range where 0 represents the CAMERA POSITION
}
impl PerspectiveCamera {
    pub fn new(position: Vector3, direction: Vector3, up: Vector3,
               fov: f32, view_bounds: (f32, f32)) 
        -> Self 
    {
        let direction  = direction.normalized();
        let horizontal = Vector3::cross(direction, up).normalized(); 
        let up         = Vector3::cross(horizontal, direction).normalized(); 
        PerspectiveCamera {
            position,
            direction,
            horizontal,
            up,
            fov,
            view_bounds
        }
    }
    pub fn shoot_at(&self, x: f32, y: f32) -> Ray {
        // NOTE This assumes that x and y have been scaled into [-1, 1]
        let z = 1.0 / f32::tan(self.fov / 2.0);

        // Generate ray from camera to the image plane.
        let ray_direction = (self.horizontal * x
                            + self.up * y
                            + self.direction * z)
                            .normalized();

        Ray::new(self.position, ray_direction)
    }
    pub fn bounds(&self) -> (f32, f32) { self.view_bounds }
}
