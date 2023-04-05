use imagestuff::{utils, raycast::{color, scene::Scene, camera::PerspectiveCamera}};

use std::convert::TryFrom;
use std::io::{self, Read, Write};
use std::fs::File;
use std::sync;
use std::path::{self, PathBuf};
use std::error;

use image::{RgbImage, Rgb};


use terminal_toys as tt;


const AA_ITERATION_COUNT: usize = 25;


type ResultError = Box<dyn error::Error>;


struct Args(path::PathBuf, usize, usize, usize, tt::SmargsResult<path::PathBuf>);

fn cli_args() -> Result<Args, String> {
    tt::smargs!(
        "Generate an image based on a scene description in JSON",
        Args(
            ("Source path of scene in JSON", ["s", "source" ], tt::SmargKind::List(1)),
            ("Width of result in pixels"   , ["w", "width"  ], tt::SmargKind::Optional("128")),
            ("Height of result in pixels"  , ["h", "height" ], tt::SmargKind::Optional("96" )),
            ("Amount of CPU threads to use", ["t", "threads"], tt::SmargKind::Optional("1"  )),
            ("Output path of the render"   , ["o", "out"    ], tt::SmargKind::Maybe)
        ),
    )
    .help_keys(vec!["help"])
    .from_env()
    .map_err(|e| e.to_string())
}

impl Args {
    fn handle_output_path(
        width: usize,
        height: usize,
        source_path: &PathBuf,
        output_path: tt::SmargsResult<PathBuf>
    ) -> Result<PathBuf, String> {
        const DEFAULT_OUTPUT_DIR: &str = "renders";

        // Make sure that output path is Ok.
        // Handle the error.
        let generate_default_output = match &output_path.0 {
            Ok(s) if s.components().next().is_none() => {
                eprint!("Received empty path. ");
                true
            },
            Err(tt::SmargsError::Dummy(e)) => {
                eprint!("Failed parsing output path: {}.", e);
                true
            },
            Err(e) => return Err(e.to_string()),
            _ok => false,
        };
        let output_path = if generate_default_output {
            // Generate the default.
            eprint!(" Generating a default...");
            let filename = utils::filename(&source_path)
                .map(|y| {
                    // TODO Confirm, that the image format can be determined by `image`(???).
                    format!("{}_{}x{}.png", y, width, height)
                }).unwrap_or_else(|| {
                    eprintln!(
                        "Failed to extract filename from '{}'",
                        source_path.display()
                    );
                    std::process::exit(1);
                });

            let y = {
                let mut p = PathBuf::new();
                p.push(DEFAULT_OUTPUT_DIR);
                p.push(filename);
                p
            };

            eprintln!("automatically setting to '{}'", y.display());

            y
        } else {
            output_path.0.unwrap()
        };

        // Check that directory for output exists before continuing with rendering
        // and potentially wasting time.
        utils::confirm_dir(output_path.parent().unwrap_or(&path::PathBuf::from(DEFAULT_OUTPUT_DIR)))?;

        Ok(output_path)
    }
}

pub fn main() -> Result<(), ResultError> {
    let Args(
        source_path,
        width,
        height,
        thread_count,
        output_path,
    ) = cli_args()?;

    // Unwrap and check output path.
    let output_path = Args::handle_output_path(width, height, &source_path, output_path)?;

    // Load view from file
    let scene = load_scene(&source_path)
        .map_err(|e| format!("Loading scene failed - {}", e))?;

    let camera = PerspectiveCamera::with_view(
        scene.fov,
        width as f32,
        height as f32
    );

    // TODO Relative paths
    // TODO Allow specifying just the output dir instead of full
    // filepath (filename still with the same old format!)
    let output_path = output_path.to_str().unwrap();

    let segment_height = height / thread_count;
    let mut img_threads = Vec::with_capacity(thread_count);

    println!("Rendering:");

    let camera = {
        sync::Arc::new(camera)
    };
    let scene = {
        sync::Arc::new(scene)
    };

    // Spawn the threads to render in
    for (i, mut progress_bar)
        in tt::ProgressBar::multiple(
                width * height,
                25,
                thread_count
            )
            .into_iter()
            .enumerate()
    {
        let arc_camera = sync::Arc::clone(&camera);
        let arc_scene = sync::Arc::clone(&scene);

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

        let mut img_vec =
            Vec::with_capacity(width * y_range.len());

        img_threads.push(
            std::thread::spawn(move || {
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
    print!("\nSaving to {} ", output_path);

    // Saving could fail for example if a previous file is open; ask to retry
    while let Err(e)
        = terminal_toys::spinner::start_spinner(
            || image.save(&output_path)
        )
    {
        println!("There was an error saving the render: {}", e);
        let mut stdout = io::stdout();
        let _ = stdout.write(b"Try saving again? [Y/n]>");
        let _ = stdout.flush();
        let mut buffer = String::new();
        let _ = io::stdin().read_line(&mut buffer);
        if buffer.starts_with('n') {
            println!("Discarding the render and exiting with error");
            // Apparently the compiler cannot infer without forcing with `as`
            // and just calling `Box::<dyn std::error::Error>::new` isn't
            // possible because Error does not implement Sized
            return Err(Box::new(e) as Box<dyn std::error::Error>)
        }
    }
    Ok(())
}

fn load_scene(filepath: &path::PathBuf) -> Result<Scene, ResultError> {
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
