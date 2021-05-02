mod vector3; // is this called a re-export?
mod objects; // is this called a re-export?

use crate::raycast::vector3::Vector3;
use crate::raycast::objects::{Sphere, Ray};

use std::env::{Args};
use image::{ImageBuffer, Rgb};

type ImgBuffer16 = ImageBuffer::<Rgb<u16>, Vec<u16>>;
type ZBounds = (f32, f32); // Represents the depth-bounds of the view-frustum

pub fn run(mut args: Args) -> Result<(), String> {
    // Image bounds
    let (width, height) = match (args.next(), args.next()) {
        (Some(a),None   ) => (a.parse().unwrap(),a.parse().unwrap()),
        (Some(a),Some(b)) => (a.parse().unwrap(),b.parse().unwrap()),
        _                 => (1024, 1024),
    };

    // Depth-bounds of the frustum
    // FIXME For some reason the frustum cuts sphere at z = 1.0 when shouldnt
    // it cut at z = -1.0 as sphere is centered at origin and has radius of 1?
    let zbs: ZBounds = (-1.5, 1.5);
    // Camera fov is 90 degrees ("fov" is named "angle" for reasons(?))
    let angle = std::f32::consts::PI / 2.0;
    let focal_point = Vector3::new(0.0,0.0,-2.0);
    // Sphere at origin
    let sphere  = Sphere::new(Vector3::new(0.5,0.0,0.0), 1.0);
    let sphere2 = Sphere::new(Vector3::new(0.0,0.5,1.0), 1.0);

    let shininess = 1.0;
    // A single light source in the scene
    let light_position = Vector3::new(0.0,1.0,-0.5);

    // Colors used:
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
