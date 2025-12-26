use std::io::Write;
use std::path;

use terminal_toys::spinner::start_spinner;

use image::io::Reader as ImageReader;
use image::DynamicImage;

/// Utility for loading an image from filepath into a DynamicImage
pub fn open_decode<P>(path: P) -> Result<DynamicImage, String>
where
    P: AsRef<path::Path>,
{
    print!("Loading image {} ", path.as_ref().to_str().unwrap());
    start_spinner(|| {
        ImageReader::open(path)
            .map_err(|e| e.to_string())?
            .decode()
            .map_err(|e| e.to_string())
    })
    .map_err(|e| e.to_string())?
}

/// If dirname directory does not exist under current directory, prompt to
/// create it. Return Ok if exists or created.
pub fn confirm_dir<T: std::convert::AsRef<path::Path>>(dirname: T) -> Result<(), String> {
    if !path::Path::exists(dirname.as_ref()) {
        print!(
            "Directory `{}` not found. Create it [y/N]? ",
            dirname.as_ref().display()
        );
        std::io::stdout().flush().map_err(|e| e.to_string())?;
        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_ok() && answer.starts_with('y') {
            return std::fs::create_dir(dirname).map_err(|e| e.to_string());
        }
        return Err("Directory not found".to_string());
    }

    Ok(())
}

/// Remove file extension and directory path from input string.
pub fn filename<T>(file: &T) -> Option<&str>
where
    T: AsRef<path::Path>,
{
    file.as_ref().file_stem().and_then(|name| name.to_str())
}

pub fn degs_to_rads(degs: f32) -> f32 {
    degs * (std::f32::consts::PI / 180.0)
}
