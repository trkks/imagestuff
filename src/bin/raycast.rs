use imagestuff::utils;
use imagestuff::raycast::{scene::Scene, camera::PerspectiveCamera};

use std::convert::{TryFrom};
use std::io::{Read};
use std::fs::{File};
use image::{RgbImage, Rgb};
use serde_json;


pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "renders";
    utils::confirm_dir(output_dir)?;

    let mut args = std::env::args();
    // Skip executable name
    args.next();
    // Load view from file
    let filepath = args.next()
        .ok_or(format!("A source file for the scene is needed"))?;

    let scene = load_scene(&filepath)
        .map_err(|e| format!("Loading scene failed - {}", e))?;

    // Image bounds
    let (width, height) = match (args.next(), args.next()) {
        (Some(a), None)    => (a.parse().unwrap(), a.parse().unwrap()),
        (Some(a), Some(b)) => (a.parse().unwrap(), b.parse().unwrap()),
        _                  => (128, 128),
    };

    let camera = PerspectiveCamera::with_view(
        scene.fov,
        width as f32,
        height as f32
    );

    // Render:
    let mut progress_bar = terminal_toys::ProgressBar::new(width * height, 25);
    progress_bar.title("Rendering");
    let image = RgbImage::from_fn(width as u32, height as u32, |ix, iy| {
        let _ = progress_bar.print_update();

        // Calculate image plane coordinates x,y so that they're in [-1, 1]
        let x: f32 = ix as f32 / width  as f32 * 2.0 - 1.0;
        // y is negated to transform from raster-space (ie. origin top left)
        // into screen-space (origin bottom left)
        let y: f32 = -(iy as f32 / height as f32 * 2.0 - 1.0);
        let ray = camera.shoot_at(x, y);

        // Shade the pixel with RGB color; 6 traces/reflections are made for
        // each intersection
        Rgb::<u8>::from(scene.trace(&ray, 6))
    });

    // Write to image file
    let result_file = format!("./{}/{}_{}x{}.png",
        output_dir, utils::filename(&filepath)?, width, height);
    print!("\nSaving to {} ", result_file);

    terminal_toys::start_spinner(|| image.save(result_file))
        // Apparently the compiler cannot infer without forcing with `as` and
        // just calling `Box::<dyn std::error::Error>::new` isn't possible
        // because Error does not implement Sized
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

fn load_scene(filepath: &str) -> Result<Scene, Box<dyn std::error::Error>> {
    let mut file = File::open(filepath)?;

    let mut contents = String::from("");
    file.read_to_string(&mut contents)?;

    let mut json: serde_json::Value =
        serde_json::from_str(&contents)?;

    Scene::try_from(&mut json)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
