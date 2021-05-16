mod general;
mod objects;
mod vector3;
mod camera;
mod ray;

use crate::utils;
use crate::raycast::{
    general::color::{self, Color},
    camera::PerspectiveCamera,
    ray::Ray,
    objects::Scene,
    vector3::Vector3,
};

use std::env::{Args};
use std::io::{Read};
use std::fs::{File};
use image::{ImageBuffer, Rgb};
use serde_json;

type ImgBuffer16 = ImageBuffer::<Rgb<u16>, Vec<u16>>;

pub fn run(mut args: Args) -> Result<(), String> {
    let output_dir = "renders";
    utils::confirm_dir(output_dir)?;

    // Load view from file
    let filepath = match args.next() {
        Some(fp) => fp,
        None => {
            eprintln!(
                "Need a path to json file describing the view to be rendered");
            std::process::exit(1);
        }
    };
    let (camera, scene) = match scene_from_json(&filepath) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error in loading objects - {}", e);
            std::process::exit(1);
        }
    };

    // Image bounds
    let (width, height) = match (args.next(), args.next()) {
        (Some(a),None   ) => (a.parse().unwrap(),a.parse().unwrap()),
        (Some(a),Some(b)) => (a.parse().unwrap(),b.parse().unwrap()),
        _                 => (128, 128),
    };

    // Amount of times to trace reflections:
    let reflection_count = 6;

    // Render:
    let image = ImgBuffer16::from_fn(width as u32, height as u32, |ix, iy| {

        // Calculate image plane coordinates x,y so that they're in [-1, 1]
        let x: f32 = ix as f32 / width  as f32 * 2.0 - 1.0;
        // y is negated to transform from raster-space (ie. origin top left)
        // into screen-space (origin bottom left)
        let y: f32 = -(iy as f32 / height as f32 * 2.0 - 1.0);
        let ray = camera.shoot_at(x, y);

        // Shade the pixel with RGB color
        shade(&scene, &ray, reflection_count).into()

    });

    // Write to image file
    let result_file = format!("./{}/{}_{}x{}.png",
        output_dir, utils::filename(&filepath)?, width, height);
    println!("Saving to {}", result_file);
    image.save(result_file).unwrap(); // TODO Handle error-result
    Ok(())
}

type ParseResult = Result<(PerspectiveCamera, Scene), String>;
fn scene_from_json(filepath: &str) -> ParseResult {
    let mut file = File::open(filepath)
        .map_err(|e| format!("Failure to open file: {}", e))?;
    let mut contents = String::from("");
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failure to read file: {}", e))?;

    let (camera, scene) = serde_json::from_str(&contents)
        .map_err(|e| format!("Failure to parse: {}", e))?;

    let camera = PerspectiveCamera::from(camera);

    Ok((camera, scene))
}

/// Recursive function that traces the ray `n` times
fn shade(scene: &Scene, ray: &Ray, n: usize) -> Color {
    // TODO is epsilon needed here?
    match scene.intersect(&ray, f32::EPSILON) {
        Some (intersect) if n > 0 => {
            // Nudge off of the surface so that ray does not re-collide
            // (see "shadow acne")
            // NOTE The "bias" (ie. normal * epsilon) seems hard to get right
            let off_intersect_surface = 
                intersect.point + (intersect.normal * 0.0001);

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
                    // TODO Calculate intensity of shadow based on intersection
                    // color's value??
                    acc
                }
            });

            // Reflections: Add color seen by reflected ray to current ray
            let reflected_ray = Ray::new(
                off_intersect_surface,
                Vector3::reflect(ray.direction(), intersect.normal)
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
        // Not same as `Vector3::reflect` for reasons:
        // wikipedia/specular_reflection
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
