use imagestuff::utils;
use imagestuff::raycast::{color, scene::Scene, camera::PerspectiveCamera};

use std::convert::{TryFrom};
use std::io::{self, Read, Write};
use std::fs::{File};
use std::sync;

use image::{RgbImage, Rgb};
use serde_json;

use terminal_toys as tt;


const AA_ITERATION_COUNT: usize = 25;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "renders";
    utils::confirm_dir(output_dir)?;

    let mut args = std::env::args();
    // Skip executable name
    args.next();
    // Load view from file
    let filepath = args.next()
        .ok_or(format!("A source file for the scene is needed"))?;

    let scene = sync::Arc::new(
        load_scene(&filepath)
            .map_err(|e| format!("Loading scene failed - {}", e))?
    );

    // Image bounds
    let (width, height) = match (args.next(), args.next()) {
        (Some(a), None)    => (a.parse().unwrap(), a.parse().unwrap()),
        (Some(a), Some(b)) => (a.parse().unwrap(), b.parse().unwrap()),
        _                  => (128, 128),
    };

    let thread_count = match args.next() {
        Some(a) => a.parse().unwrap(),
        None    => 4,
    };

    let camera = sync::Arc::new(
        PerspectiveCamera::with_view(
            scene.fov,
            width as f32,
            height as f32
        )
    );

    println!("Rendering:");

    let segment_height = height / thread_count;
    let mut img_threads = Vec::with_capacity(thread_count);

    // Spawn the threads to render in
    for (i, mut progress_bar)
        in tt::ProgressBar::multiple(width * height, 25, thread_count)
            .into_iter()
            .enumerate()
    {
        let arc_camera = sync::Arc::clone(&camera);
        let arc_scene = sync::Arc::clone(&scene);

        img_threads.push(
            std::thread::spawn(move || {
                // Every pixel in segment counts towards progress
                progress_bar.title(&format!("  Thread #{} progress", i + 1));

                let y_range = {
                    // Iterate the coordinates in image segments
                    let start = i * segment_height;
                    let mut end = start + segment_height;
                    // The last created thread takes the remaining rows as well
                    if i == thread_count - 1 {
                        end += height % thread_count;
                    }
                    start..end
                };

                let mut img_vec = Vec::with_capacity(width * y_range.len());
                for iy in y_range {
                    for ix in 0..width {
                        img_vec.push(
                            shade_pixel(
                                ix, iy,
                                width, height,
                                &arc_camera, &arc_scene
                            )
                        );
                        progress_bar.lap()
                            .expect("Progress bar print failure");
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

    // Move command line cursor to bottom of progress bars
    print!("\x1b[{}B", thread_count);

    // Use `from_fn` instead of `from_vec` in order to not manually handle
    // unwrapping the Subpixel -associated-type
    let image = RgbImage::from_fn(
        width as u32, height as u32,
        |ix, iy| img_combined[iy as usize * width + ix as usize]
    );

    // Write to image file
    let result_file = format!("./{}/{}_{}x{}.png",
        output_dir, utils::filename(&filepath)?, width, height);
    print!("\nSaving to {} ", result_file);

    // Saving could fail for example if a previous file is open; ask to retry
    while let Err(e)
        = terminal_toys::spinner::start_spinner(|| image.save(&result_file))
    {
        println!("There was an error saving the render: {}", e);
        let mut stdout = io::stdout();
        let _ = stdout.write(b"Try saving again? [Y/n]>");
        let _ = stdout.flush();
        let mut buffer = String::new();
        let _ = io::stdin().read_line(&mut buffer);
        if buffer.starts_with("n") {
            println!("Discarding the render and exiting with error");
            // Apparently the compiler cannot infer without forcing with `as`
            // and just calling `Box::<dyn std::error::Error>::new` isn't
            // possible because Error does not implement Sized
            return Err(Box::new(e) as Box<dyn std::error::Error>)
        }
    }
    Ok(())
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

fn shade_pixel(
    ix: usize,
    iy: usize,
    width: usize,
    height: usize,
    camera: &sync::Arc<PerspectiveCamera>,
    scene: &sync::Arc<Scene>,
) -> Rgb<u8> {
    let mut color = color::consts::BLACK;

    // Anti-aliasing: sample each pixel in some pattern and return average
    // TODO Separate AA into a general function; AA works on a pixel:
    // 1) get location and size of a pixel (input: rectangle)
    // 2) shoot rays into these bounds (output: coordinates)
    for _ in 0..AA_ITERATION_COUNT {
        let (tx, ty) = (
            rand::random::<f32>(),
            rand::random::<f32>(),
        );
        // Calculate image plane coordinates x,y so that they're in [-1, 1]
        let x: f32 = (ix as f32 + tx) / width as f32 * 2.0 - 1.0;
        // y is negated to transform from raster-space (ie. origin top left)
        // into screen-space (origin bottom left)
        let y: f32 = -((iy as f32 + ty) / height as f32 * 2.0 - 1.0);

        let ray = camera.shoot_at(x, y);

        // Shade the pixel with RGB color; 6 traces/reflections are made for
        // each intersection
        color += &scene.trace(&ray, 6);
    }

    Rgb::<u8>::from(color * (1.0 / AA_ITERATION_COUNT as f32))
}
