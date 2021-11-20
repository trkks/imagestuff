use std::path;

use imagestuff::utils;


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
    while let Ok(n) = stdin.read_line(&mut input) {
        if n > 0 {
            get_path(&input.trim(), &mut paths);
            input.clear();
        } else {
            break;
        }
    }
    return if paths.len() > 0 { Some(paths) } else { None };
}


fn cl_args() -> Option<Vec<path::PathBuf>> {
    // TODO Use terminal_toys::Smargs
    let mut args = std::env::args();
    // Skip executable name
    args.next();
    // Get the target filepaths
    let mut paths = Vec::new();
    while let Some(arg) = args.next() {
        get_path(&arg, &mut paths);
    }

    // Load the images
    //let frames = paths.iter().map(utils::open_decode);
    return if paths.len() > 0 { Some(paths) } else { None };
}

fn get_path(x: &str, paths: &mut Vec<path::PathBuf>) {
    if let Ok(path) = std::fs::canonicalize(&x) {
        paths.push(path);
    } else {
        eprintln!("Not a valid path input: '{}'", x);
        std::process::exit(1);
    }
}

fn process(paths: Vec<path::PathBuf>) {
    todo!()
}
