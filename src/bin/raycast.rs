use imagestuff::utils;
use imagestuff::raycast::{scene::Scene, camera::PerspectiveCamera};

use std::convert::{TryFrom};
use std::io::{Read};
use std::fs::{File};
use image::{ImageBuffer, Rgb};
use serde_json;

use terminal_toys as tt;


type ImgBuffer16 = ImageBuffer::<Rgb<u16>, Vec<u16>>;

pub fn main() -> Result<(), String> {
    let output_dir = "renders";
    utils::confirm_dir(output_dir)?;

    let mut args = std::env::args();
    // Skip executable name
    args.next();
    // Load view from file
    let filepath = args.next()
        .ok_or(format!("A source file for the scene is needed"))?;

    let scene = std::sync::Arc::new(
        load_scene(&filepath)
            .map_err(|e| format!("Loading scene failed - {}", e))?
    );

    // Image bounds
    let (width, height) = match (args.next(), args.next()) {
        (Some(a), None)    => (a.parse().unwrap(), a.parse().unwrap()),
        (Some(a), Some(b)) => (a.parse().unwrap(), b.parse().unwrap()),
        _                  => (128, 128),
    };

    let camera = std::sync::Arc::new(
        PerspectiveCamera::with_view(
            scene.fov,
            width as f32,
            height as f32
        )
    );

    println!("Rendering:");
    let thread_count = 4;
    let segment_height = height / thread_count;
    let mut img_threads = Vec::with_capacity(thread_count);
    // Spawn the threads to render in
    for i in 0..thread_count {
        let arc_camera = std::sync::Arc::clone(&camera);
        let arc_scene = std::sync::Arc::clone(&scene);
        img_threads.push(
            std::thread::spawn(move || {
                // Every pixel in segment counts towards progress
                let mut progress_bar =
                    tt::ProgressBar::new(width * segment_height, 25);
                progress_bar.title(&format!("  Thread #{} progress", i + 1));

                let mut img_vec = Vec::with_capacity(width * segment_height);
                // Iterate the coordinates in image segment
                let start_coord = i * segment_height;
                for iy in start_coord..start_coord + segment_height {
                    for ix in 0..width {
                        // Calculate image plane coordinates x,y so that
                        // they're in [-1, 1]
                        let x: f32 = ix as f32 / width as f32 * 2.0 - 1.0;
                        // y is negated to transform from raster-space (ie.
                        // origin top left) into screen-space (origin bottom
                        // left)
                        let y: f32 = -(iy as f32 / height as f32 * 2.0 - 1.0);
                        let ray = arc_camera.shoot_at(x, y);

                        img_vec.push(
                            // Shade the pixel with RGB color; 6
                            // traces/reflections are made for each
                            // intersection
                            Rgb::<u16>::from(arc_scene.trace(&ray, 6))
                        );

                        let _ = progress_bar.print_update_row(i + 1).unwrap();
                    }
                }
                // Return the rendered pixels in segment
                img_vec
            })
        );
    }

    // Wait for rendering threads to finish and combine the rendered segments
    // in order
    let mut img_combined = Vec::with_capacity(width * height);
    for t in img_threads {
        img_combined.append(&mut t.join().unwrap());
    }
    // Use `from_fn` instead of `from_vec` in order to not manually handle
    // unwrapping the Subpixel -associated-type
    let image = ImgBuffer16::from_fn(
        width as u32, height as u32,
        |ix, iy| img_combined[iy as usize * width + ix as usize]
    );

    // Write to image file
    let result_file = format!("./{}/{}_{}x{}.png",
        output_dir, utils::filename(&filepath)?, width, height);
    print!("\nSaving to {} ", result_file);

    tt::start_spinner(|| image.save(result_file))
        .map_err(|e| format!("Failed to save {}", e))
}

fn load_scene(filepath: &str) -> Result<Scene, String> {
    let mut file = File::open(filepath).map_err(|e| e.to_string())?;

    let mut contents = String::from("");
    file.read_to_string(&mut contents).map_err(|e| e.to_string())?;

    let mut json: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| e.to_string())?;

    Scene::try_from(&mut json).map_err(|e| e.to_string())
}
