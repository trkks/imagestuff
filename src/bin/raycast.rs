use std::io::{self, Write};
use std::path;

use image::RgbImage;
use terminal_toys as tt;

use imagestuff::{utils, raycast::{scene, camera, raycaster}};


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
        source_path: &path::PathBuf,
        output_path: tt::SmargsResult<path::PathBuf>
    ) -> Result<path::PathBuf, String> {
        const DEFAULT_OUTPUT_DIR: &str = "renders";

        // Make sure that output path is Ok.
        // Handle the error.
        let generate_default_output = match &output_path.0 {
            Ok(s) if s.components().next().is_none() => {
                eprint!("No output path received.");
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
                let mut p = path::PathBuf::new();
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

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args(
        source_path,
        width,
        height,
        thread_count,
        output_path,
    ) = cli_args().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    // Unwrap and check output path.
    // TODO Relative paths
    // TODO Allow specifying just the output dir instead of full filepath
    // (filename still with the same old format!)
    let output_path = Args::handle_output_path(width, height, &source_path, output_path)?;

    let raycaster = {
        let scene = scene::Scene::from_file(&source_path)?;
        let camera = camera::PerspectiveCamera::with_view(scene.fov, width, height);
        raycaster::Raycaster { scene, camera }
    };

    let image = RgbImage::from_vec(width as u32, height as u32, raycaster.render_rgb_flat(thread_count))
        .unwrap();

    // Write to image file
    print!("\nSaving to {} ", output_path.display());

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
