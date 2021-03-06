use std::path;
use std::io::Write;

use terminal_toys::spinner::start_spinner;

use image::io::{Reader as ImageReader};
use image::DynamicImage;


/// Utility for loading an image from filepath into a DynamicImage
pub fn open_decode<P>(path: P) -> Result<DynamicImage, String>
where
    P: AsRef<path::Path>,
{
    print!("Loading image {} ", path.as_ref().to_str().unwrap());
    start_spinner(|| ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())
    )
}

/// If dirname directory does not exists under current directory, prompt to
/// create it. Return True if exists or created
pub fn confirm_dir(dirname: &str) -> Result<(), String> {
    let dir = dirname;

    if !path::Path::new(&dir).exists() {
        print!("Directory `{}` not found. Create it [y/N]? ", dir);
        std::io::stdout().flush().map_err(|e| e.to_string())?;
        let mut answer = String::new();
        if let Ok(_) = std::io::stdin().read_line(&mut answer) {
            if answer.starts_with("y") {
                return std::fs::create_dir(dir).map_err(|e| e.to_string())
            }
        }
        return Err(format!("Directory not found"))
    }

    Ok(())
}

/// Remove file extension and directory path from input string.
pub fn filename<'a, T>(file: &'a T) -> Option<&'a str>
where
    T: AsRef<path::Path>,
{
    file.as_ref().file_stem().and_then(|name| name.to_str())
}

pub fn degs_to_rads(degs: f32) -> f32 {
    degs * (std::f32::consts::PI / 180.0)
}
