mod general; // is this called a re-export?
mod objects; // is this called a re-export?
mod vector3; // is this called a re-export?

use crate::raycast::{ // is there a better way?
    general::{color, PerspectiveCamera, Intersection, Ray},
    objects::{Sphere, Scene},
    vector3::Vector3,
};

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

    let camera = PerspectiveCamera::new(
        Vector3::new(0.0, 0.0,-50.0), // position
        Vector3::new(0.0, 0.0, 1.0),  // direction
        Vector3::new(0.0,-1.0, 0.0),  // up FIXME Why is this "reversed" :/?
        std::f32::consts::PI / 4.0,   // fov
        (0.0, f32::MAX)               // near and far bounds of view
    );

    // Make a scene
    let scene = Scene::from(vec![
            Sphere::new(Vector3::new( 0.5,-0.3, 0.2), 0.5),
            Sphere::new(Vector3::new(-0.5,-0.5, 0.2), 0.3),
            Sphere::new(Vector3::new( 0.0, 0.5, 0.0), 0.4),
        ]
    );

    // FIXME set this to 0.0 and go study some more
    let shininess = 4.0;
    // A single light source in the scene
    //FIXME increasing distance amplifies the light
    let light_position = Vector3::new(0.0, 1.0,-1.0);

    // Colors used:
    let specular         = color::consts::WHITE;
    let background_color = color::consts::BLACK;

    // Render:
    let image = ImgBuffer16::from_fn(width as u32, height as u32, |ix, iy| {

        // Calculate image plane coordinates x,y so that they're in [-1, 1]
        let x: f32 = (ix as f32 / width  as f32) * 2.0 - 1.0;  
        let y: f32 = (iy as f32 / height as f32) * 2.0 - 1.0;  
        let ray = camera.shoot_at(x, y);

        let shade = |intersection: Intersection| {
            // Run the raytracing: 
            // TODO Feed the scene to a function for intersecting recursively
            // for n iterations or until intersection returns None. Ray
            // direction is the reflection and ray origin is the intersection
            // point
            // Shade the pixel on sphere
            // FIXME tracing not working (because not actually "recursive"?)
            (0..1).fold((intersection, background_color), |(intr, col), _| {
                let normal = intr.normal;
                let surface_to_light = (light_position - intr.point)
                                       .normalized();

                // Diffuse shading:
                let diffuse_amount = surface_to_light.dot(normal)
                                     .max(0.0);

                // Specular shading: 
                //TODO Is refl normalized after operation? (see wikipedia ^R)
                let reflection = normal * surface_to_light.dot(normal) * 2.0
                                 - surface_to_light;
                let specular_amount = reflection.dot(-ray.direction())
                                      .max(0.0)
                                      .powf(shininess);

                let next_ray = Ray::new(intr.point, reflection);
                let next_inter = scene.intersect(&next_ray, 0.0);
                if let Some(intr2) = next_inter {
                    let matcol = intr.material.color;
                    (intr2, col + (matcol * diffuse_amount)
                    + (specular * specular_amount))
                } else {
                    let matcol = intr.material.color;
                    (intr, col + (matcol * diffuse_amount)
                    + (specular * specular_amount))
                }
            }).1
        };
        
        // Select the closest intersection for rendering
        let color = match scene.intersect(&ray, camera.bounds().0) {
            Some(intersection) => shade(intersection),
            _ => background_color // Shade the pixel on background
        };

        color.into()

    });

    // Write to image file
    image.save("raycast_sphere.png").unwrap(); // TODO Handle error-result

    Ok(())
}
