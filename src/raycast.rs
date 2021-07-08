mod general;
mod objects;
mod vector3;
mod matrix;
mod camera;
mod ray;

use crate::utils;
use crate::raycast::{
    general::{Intersect, color::{self, Color} },
    camera::PerspectiveCamera,
    ray::Ray,
    objects::Scene,
    vector3::{Vector3, UnitVector3},
};

use std::convert::{TryFrom};
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

    // Render:
    let aspect_ratio = width as f32 / height as f32;
    let mut progress_bar = terminal_toys::ProgressBar::new(width * height, 25);
    progress_bar.title("Rendering");
    let image = ImgBuffer16::from_fn(width as u32, height as u32, |ix, iy| {
        let _ = progress_bar.print_update();

        // Calculate image plane coordinates x,y so that they're in [-1, 1]
        let x: f32 = ix as f32 / width  as f32 * 2.0 - 1.0;
        // y is negated to transform from raster-space (ie. origin top left)
        // into screen-space (origin bottom left)
        let y: f32 = -(iy as f32 / height as f32 * 2.0 - 1.0);
        let ray = camera.shoot_at(x, y, aspect_ratio);

        // Shade the pixel with RGB color; 6 traces/reflections are made for
        // each intersection
        shade(&scene, &ray, 6).into()

    });

    // Write to image file
    let result_file = format!("./{}/{}_{}x{}.png",
        output_dir, utils::filename(&filepath)?, width, height);
    print!("\nSaving to {} ", result_file);

    terminal_toys::start_spinner(|| image.save(result_file))
        .map_err(|e| format!("Failed to save {}", e))
}

type ParseResult = Result<(PerspectiveCamera, Scene), String>;
fn scene_from_json(filepath: &str) -> ParseResult {
    let mut file = File::open(filepath)
        .map_err(|e| format!("Failure to open file: {}", e))?;
    let mut contents = String::from("");
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failure to read file: {}", e))?;

    let mut data: serde_json::value::Value = serde_json::from_str(&contents)
        .map_err(|e| format!("Failure with JSON: {}", e))?;
    let camera = PerspectiveCamera::try_from(data["camera"].take())
        .map_err(|e| format!("Failure with camera: {}", e))?;
    let scene = Scene::try_from(data["scene"].take())
        .map_err(|e| format!("Failure with scene: {}", e))?;

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

            let color = scene.lights.iter().fold(
                scene.ambient_color, |acc, light| {

                let (distance_to_light, surface_to_light) = {
                    let v = light.position - intersect.point;
                    (v.length(), v.normalized())
                };

                // Shadows:
                let ray_to_light = Ray {
                    origin: off_intersect_surface,
                    direction: surface_to_light,
                };
                // If intersection point is hit by the light, color it
                if scene.intersect(&ray_to_light, f32::EPSILON).is_none() {
                    // Shading model from:
                    // http://www.cs.cornell.edu/courses/cs4620/2014fa/lectures/05rt-shading.pdf
                    let light_intensity = light.intensity / distance_to_light;
                    let bisector = {
                        let v: Vector3 = (-ray.direction).into();
                        let w: Vector3 = surface_to_light.into();
                        // From wikipedia "Phong reflection model":
                        //let w: Vector3 = 2.0 * surface_to_light.dot(&intersect.normal) * Vector3::from(intersect.normal) - Vector3::from(surface_to_light);
                        (v + w).normalized()
                    };

                    acc 
                    + intersect.material.color
                    * light_intensity
                    * f32::max(0.0, intersect.normal.dot(&surface_to_light))
                    + light.color
                    * light_intensity
                    * f32::max(0.0, intersect.normal.dot(&bisector))
                        .powi(intersect.material.shininess)
                } else {
                    // Ignore light at this point
                    acc
                }
            });

            // Reflections: Add color seen by reflected ray to current ray
            let reflected_ray = Ray {
                origin: off_intersect_surface,
                direction: ray.direction.reflect(&intersect.normal)
            };
            // Recursive call:
            return color
                + shade(
                    &scene,
                    &reflected_ray,
                    n - 1
                )
        },
        // Shade with ambient color and end recursion
        _ => scene.ambient_color
    }
}
