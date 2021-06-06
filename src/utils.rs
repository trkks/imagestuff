use std::io::Write;
use image::io::{Reader as ImageReader};
use image::DynamicImage;


// Utility for loading an image from filepath into a DynamicImage
pub fn open_decode(file: &str) -> Result<DynamicImage, String> {
    println!("Loading image {}", &file);
    ImageReader::open(&file)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())
}

// If dirname directory does not exists under current directory, prompt to
// create it. Return True if exists or created
pub fn confirm_dir(dirname: &str) -> Result<(), String> {
    let dir = dirname;

    if !std::path::Path::new(&dir).exists() {
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

// Remove file extension and directory path from input string
pub fn filename(file: &str) -> Result<&str,String> {
    if let Some(name) = std::path::Path::new(file).file_stem() {
        if let Some(s) = name.to_str() {
            return Ok(s)
        }
    }
    Err(format!("Bad filename: {}",file))
}
