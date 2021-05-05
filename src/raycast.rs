mod general; // is this called a re-export?
mod objects; // is this called a re-export?
mod vector3; // is this called a re-export?

use crate::raycast::{ // is there a better way?
    general::{color::{self, Color}, PerspectiveCamera, Ray, Material},
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
        _                 => (128, 128),
    };

    // Using the right hand rule/basis idk
    let camera = PerspectiveCamera::new(
        Vector3::new(0.0, 0.0,-1.0), // position
        Vector3::new(0.0, 0.0, 1.0), // direction
        Vector3::new(0.0, 1.0, 0.0), // up FIXME Why is this "reversed" :/?
        std::f32::consts::PI / 2.0,  // fov
        (0.0, f32::MAX)              // near and far bounds of view
    );

    // Make a scene TODO Parse from external source
    let scene = Scene::from(
        color::consts::BLACK,
        vec![
            Sphere::new(Vector3::new( 0.45,-0.30, 1.4), 0.5,
                 Material { color: color::consts::RED }),
            Sphere::new(Vector3::new(-0.25,-0.35, 0.6), 0.3,
                 Material { color: color::consts::GREEN }),
            Sphere::new(Vector3::new(-0.20, 0.40, 1.0), 0.4,
                 Material { color: color::consts::BLUE }),
        ],
        vec![
            // This light casts the shadow of green sphere onto red sphere
            Light {
                position:  Vector3::new(-0.75,-0.35,-1.4),
                _direction: None,
                color: color::consts::WHITE,
                intensity: 1.0,
            },
            Light {
                position: Vector3::new( 0.5, 1.5, 3.0),
                _direction: None,
                color: color::consts::NEON_PINK,
                intensity: 2.0,
            },
        ],
    );


    // Amount of times to trace reflections:
    let reflection_count = 6;

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
fn shade(scene: &Scene, ray: &Ray, n: usize) -> Color {
    // TODO is epsilon needed here?
    match scene.intersect(&ray, f32::EPSILON) {
        Some (intersect) if n > 0 => {
            // Nudge off of the surface so that ray does not re-collide
            // (see "shadow acne")
            let off_intersect_surface = 
                intersect.point + (intersect.normal * f32::EPSILON);

            let color = scene.lights().iter().fold(
                color::consts::BLACK, |acc, light| {

                let surface_to_light = (light.position - intersect.point)
                                       .normalized();

                // Shadows:
                let ray_to_light = Ray::new(
                    off_intersect_surface,
                    surface_to_light,
                );
                if scene.intersect(&ray_to_light, f32::EPSILON).is_none() {
                    // intersection point is hit by the light, so color it
                    acc 
                    + phong(
                        intersect.material.color, 
                        light.color_to_target(intersect.point),
                        surface_to_light,
                        intersect.normal,
                        -ray.direction()
                    )
                } else {
                    // ignore light at this point
                    acc
                }
            });

            // Reflections: Add color seen by reflected ray to current ray
            let reflected_ray = Ray::new(
                off_intersect_surface,
                reflect(ray.direction(), intersect.normal)
            );
            // Recursive call:
            return color + shade(&scene, &reflected_ray, n-1)
        },
        // Shade with ambient color and end recursion
        _ => scene.ambient_color()
    }
}

/// Phong reflection; color is the sum of diffuse and specular by light
fn phong(diffuse: Color, 
         specular: Color,
         surface_to_light: Vector3,
         normal: Vector3,
         to_viewer: Vector3) -> Color {
    // FIXME set this to 0.0 and decide go study some more
    let shininess = 7.0; // TODO Move this into Material?;

    // From wikipedia: "each term should only be included if the term's dot
    // product is positive"

    // Dot product of the two terms:
    // Diffuse
    let diffuse_term = surface_to_light.dot(normal);
    // Specular
    let reflection = 
        // Not same as `fn reflect` for reasons: wikipedia/specular_reflection
        2.0 * surface_to_light.dot(normal) * normal - surface_to_light; 
    let specular_term = reflection.dot(to_viewer);

    // Diffuse shading:
    diffuse
        * if diffuse_term <= 0.0 {
            0.0
        } else {
            diffuse_term
        }
    // Specular shading:
    // ...continuing: "Additionally, the specular term should only be
    // included if the dot product of the diffuse term is positive"
    + specular
        * if specular_term <= 0.0 || diffuse_term <= 0.0 {
            0.0
        } else {
            specular_term.powf(shininess)
        }
}

// NOTE d and n must be normalized before calling this
fn reflect(d: Vector3, n: Vector3) -> Vector3 {
    d - 2.0 * d.dot(n) * n
}
