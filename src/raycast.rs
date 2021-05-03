mod general; // is this called a re-export?
mod objects; // is this called a re-export?
mod vector3; // is this called a re-export?

use crate::raycast::{ // is there a better way?
    general::{color::{self, Color}, PerspectiveCamera, Intersection, Ray},
    objects::{Sphere, Scene, Light},
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
    let scene = Scene::from(
        color::consts::BLACK,
        vec![
            Sphere::new(Vector3::new( 0.4,-0.2, 0.2), 0.5),
            Sphere::new(Vector3::new(-0.5,-0.5, 0.4), 0.3),
            Sphere::new(Vector3::new(-0.3, 0.5, 0.0), 0.4),
        ],
        // FIXME changing light positions seems to have no
        // effect on amount of light
        vec![
            Light {
                position: Vector3::new(0.0, 0.0,-1.0),
                _direction: None,
                color: Color::new(0.8, 0.8, 0.8),
            },
            Light {
                position: Vector3::new(-1.5,-0.5, 4.0),
                _direction: None,
                color: Color::new(0.0, 0.8, 0.4),
            },
        ],
    );


    // Amount of times to trace reflections:
    let reflection_count = 1;

    // Render:
    let image = ImgBuffer16::from_fn(width as u32, height as u32, |ix, iy| {

        // Calculate image plane coordinates x,y so that they're in [-1, 1]
        let x: f32 = (ix as f32 / width  as f32) * 2.0 - 1.0;  
        let y: f32 = (iy as f32 / height as f32) * 2.0 - 1.0;  
        let ray = camera.shoot_at(x, y);

        // Shade the pixel with RGB color
        shade(&scene, &ray, reflection_count).into()

    });

    // Write to image file
    image.save("raycast_sphere.png").unwrap(); // TODO Handle error-result

    Ok(())
}

/// Recursive function that traces the ray `n` times
fn shade(scene: &Scene, ray: &Ray, n: usize) -> Color
{
    // Recursion end
    if n <= 0 { return scene.ambient_color() }

    //TODO Is a reflection normalized by default? (see wikipedia ^R)
    let reflect = |d: Vector3, n: Vector3| -> Vector3 {
        (n * 2.0 * d.dot(n) - d) // wikipedia.phong_reflection_model
        .normalized()
    };

    // Phong reflection; color is the sum of diffuse and specular by lights
    let phong = |intr: Intersection, lights: &Vec<Light>| -> Color {
        // FIXME set this to 0.0 and decide go study some more
        let shininess = 3.0; // TODO Move this into Material?

        // TODO choose proper starting color: black, ambient or what?
        lights.iter().fold(scene.ambient_color(), |acc, light| {
            // NOTE this might not meet "the standards"
            let specular = light.color;

            let normal = intr.normal;
            let surface_to_light = (light.position - intr.point)
                                   .normalized();

            // Diffuse shading:
            let diffuse_amount = surface_to_light.dot(normal)
                                 .max(0.0);

            // Specular shading:
            let reflection = reflect(surface_to_light, normal);
            let specular_amount = reflection.dot(-ray.direction())
                                  .max(0.0)
                                  .powf(shininess);

            acc
            + (intr.material.color * diffuse_amount)
            + (specular * specular_amount)
        })
    };

    // TODO Where to store min_t?
    match scene.intersect(&ray, 0.0) {
        // Shade with reflections in the scene
        Some (intersection) => {
            let reflected_ray = Ray::new(
                intersection.point,
                reflect(intersection.point.normalized(), intersection.normal)
            );

            // Recursive call
            // FIXME tracing not working because not properly "recursive"?
            return phong(intersection, &scene.lights())
                   + shade(&scene, &reflected_ray, n-1)
        },
        // Shade with ambient color
        _ => scene.ambient_color()
    }
}
