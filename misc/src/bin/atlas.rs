use std::path;

use utils;


const HELP: &str =
r#"Description:
    Atlas glues images into an array representation.
Syntax:
    atlas [path] --width int --height int --output path
Examples:
    # From three images in current working directory, create a 1 by 3 image with the name 'anim_frames.png'
    atlas anim_frame0.png anim_frame1.png anim_frame2.png --width 1 --output anim_frames.png
    # Create an image from all the files ending with '*.png' found in the './frames' directory
    atlas frames/*.png
"#;

pub fn main() {
    if let Some(paths) = piped_input().or_else(cl_args) {
        println!("{:?}", paths);
        process(paths);
    } else {
        println!("{}", HELP);
    }
}

fn piped_input() -> Option<Vec<path::PathBuf>> {
    let stdin = std::io::stdin();
    let mut input = String::new();
    let mut paths = Vec::new();
    // FIXME if nothing is piped, blocks forever
    while let Ok(n) = stdin.read_line(&mut input) {
        if n > 0 {
            get_path(input.trim(), &mut paths);
            input.clear();
        } else {
            break;
        }
    }
    if !paths.is_empty() { Some(paths) } else { None }
}


fn cl_args() -> Option<Vec<path::PathBuf>> {
    // TODO Use terminal_toys::Smargs
    let mut args = std::env::args();
    // Skip executable name
    args.next();
    // Get the target filepaths
    let mut paths = Vec::new();
    for arg in args {
        get_path(&arg, &mut paths);
    }

    // Load the images
    //let frames = paths.iter().map(utils::open_decode);
    if !paths.is_empty() { Some(paths) } else { None }
}

fn get_path(x: &str, paths: &mut Vec<path::PathBuf>) {
    if let Ok(path) = std::fs::canonicalize(x) {
        paths.push(path);
    } else {
        eprintln!("Not a valid path input: '{}'", x);
        std::process::exit(1);
    }
}

fn process(paths: Vec<path::PathBuf>) {
    println!("{:?}", paths);
    let user_output = std::env::args().nth(1);
    // The resulting image will be sized by the maximum dimensions times the
    // number of input images
    // TODO Should the dimensions be decided based on the first image?
    let (max_width, max_height) = {
        // TODO This could possibly be done simpler by scanning?
        let (widths, heights): (Vec<u32>,Vec<u32>) = paths.iter()
            .map(|x| image::image_dimensions(x).unwrap())
            .unzip();
        (*widths.iter().max().unwrap(), *heights.iter().max().unwrap())
    };

    // TODO organize width and height according to the wanted dimensions of the
    // resulting atlas
    let mut result = image::ImageBuffer::new(
        max_width * paths.len() as u32,
        max_height
    );

    // Combine the images
    for (i, path) in paths.iter().enumerate() {
        let image = utils::open_decode(&path).unwrap();
        image::imageops::replace(&mut result, &image, i as u32 * max_width, 0);
    }

    // Save the result to the wanted file if given
    // TODO Should the decided dimensions (size of a single "frame" and the
    // table-configuration) be included to the filename?
    let output_path = user_output
        .unwrap_or(String::from("./pics/atlas_result.png"));
    println!("Saving to {}", output_path);
    if let Err(e) = result.save(output_path) {
        eprintln!("Failed to save: {}", e);
        std::process::exit(1);
    }
}
