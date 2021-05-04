mod general; // is this called a re-export?
mod objects; // is this called a re-export?
mod vector3; // is this called a re-export?

use crate::raycast::{ // is there a better way?
    general::{color::{self, Color}, PerspectiveCamera, Intersection, Ray, Material},
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
        Vector3::new(0.0, 0.0,-10.0), // position
        Vector3::new(0.0, 0.0, 1.0),  // direction
        Vector3::new(0.0,-1.0, 0.0),  // up FIXME Why is this "reversed" :/?
        std::f32::consts::PI / 2.0,   // fov
        (0.0, f32::MAX)               // near and far bounds of view
    );

    // Make a scene TODO Parse from external source
    let scene = Scene::from(
        color::consts::BLACK,
        vec![
            Sphere::new(Vector3::new( 0.30,-0.30, 0.4), 0.5,
                 Material { color: color::consts::RED }),
            Sphere::new(Vector3::new(-0.30,-0.35,-0.4), 0.3,
                 Material { color: color::consts::GREEN }),
            Sphere::new(Vector3::new(-0.35, 0.40, 0.0), 0.4,
                 Material { color: color::consts::BLUE }),
        ],
        vec![
            Light {
                position: Vector3::new(0.0, 1.0,-2.0),
                _direction: None,
                color: color::consts::WHITE,
                intensity: 1.0,
            },
            Light {
                position: Vector3::new(-1.5,-0.5, 4.0),
                _direction: None,
                color: color::consts::NEON_PINK,
                intensity: 5.0,
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
    // TODO Where to store min_t? Should min_t be epsilon rather than "0"?
    match scene.intersect(&ray, 0.0) {
        // Shade with reflections in the scene
        Some (intersect) if n > 0 => {
            let reflected_ray = Ray::new(
                // Nudge off of the surface so that ray does not re-collide
                // (see "shadow acne")
                intersect.point + f32::EPSILON * intersect.normal,
                reflect(intersect.point.normalized(), intersect.normal)
            );

            // TODO choose proper base color: black, ambient or what?
            return phong(intersect, &scene.lights(),
                         scene.ambient_color(), -ray.direction())
                   // Recursive call:
                   // FIXME Major "tearing" on reflections
                   // Seems that reflections are "reversed"; problem with
                   // camera/the space??? Solving needs studying???
                   + shade(&scene, &reflected_ray, n-1)
        },
        // Shade with ambient color and end recursion
        _ => scene.ambient_color()
    }
}

/// Phong reflection; color is the sum of diffuse and specular by lights
fn phong(intr: Intersection, lights: &Vec<Light>,
         base: Color,        to_viewer: Vector3)
        -> Color
{
    // FIXME set this to 0.0 and decide go study some more
    let shininess = 3.0; // TODO Move this into Material?

    lights.iter().fold(base, |acc, light| {
        // NOTE this selection of color might not "meet the standards"
        let specular_color = light.color_to_target(intr.point);

        let normal = intr.normal;
        let surface_to_light = (light.position - intr.point)
                               .normalized();

        // From wikipedia: "each term should only be included if the term's dot
        // product is positive"

        // Dot product of the two terms:
        // Diffuse
        let diffuse_term = surface_to_light.dot(normal);
        // Specular
        let reflection = reflect(surface_to_light, normal);
        let specular_term = reflection.dot(to_viewer);

        return acc
            // Diffuse shading:
            + intr.material.color
                * if diffuse_term <= 0.0 {
                    0.0
                } else {
                    diffuse_term
                }
            // Specular shading:
            // ...continuing: "Additionally, the specular term should only be
            // included if the dot product of the diffuse term is positive"
            + specular_color
                * if specular_term <= 0.0 || diffuse_term <= 0.0 {
                    0.0
                } else {
                    specular_term.powf(shininess)
                }
    })
}

// NOTE d and n must be normalized before calling this
fn reflect(d: Vector3, n: Vector3) -> Vector3 {
        2.0 * d.dot(n) * n - d // wikipedia/specular_reflection
}
