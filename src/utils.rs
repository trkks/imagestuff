use std::io::Write;
use image::io::{Reader as ImageReader};
use image::{Rgb, DynamicImage};


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

// Get ascii character that looks like the brightness of the pixel
pub fn pixel_to_ascii(pixel: Rgb<u16>) -> char {
    // Divide by more (0.2) than count (3) to make slightly darker
    let brightness = (pixel[0] as f32 / u16::MAX as f32 +
                      pixel[1] as f32 / u16::MAX as f32 +
                      pixel[2] as f32 / u16::MAX as f32) / 3.2;

    if 0.875 <= brightness { return '#'; }
    if 0.75  <= brightness { return '@'; }
    if 0.625 <= brightness { return '0'; }
    if 0.5   <= brightness { return 'o'; }
    if 0.375 <= brightness { return '='; }
    if 0.25  <= brightness { return '~'; }
    if 0.125 <= brightness { return '-'; }    
                             return ' ';
}

pub fn half_lerp(p1: Rgb<u16>, p2: Rgb<u16>) -> [u16;3] {
    // Linear interpolation aped from wikipedia
    // Normalize and subtract and divide by 2 and add p1 and return 
    [p1[0] + ((p2[0] as f32 - p1[0] as f32) as u16 >> 1),
     p1[1] + ((p2[1] as f32 - p1[1] as f32) as u16 >> 1),
     p1[2] + ((p2[2] as f32 - p1[2] as f32) as u16 >> 1)]
}
