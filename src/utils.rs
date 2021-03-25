use std::io::Write;
use image::io::Reader as ImageReader;
use image::{Rgb, DynamicImage};


// Utility for loading an image from filepath into a DynamicImage
pub fn open_decode(file: &str) -> Result<DynamicImage, String> {
    println!("Loading image {}", &file);
    match ImageReader::open(&file) {
        Ok(reader) => reader.decode().map_err(|e| format!("{}", e)), 
        Err(msg)   => Err(format!("{}", msg)),
    }
}

// If dirname directory does not exists under current directory, prompt to
// create it. Return True if exists or created
pub fn confirm_dir(dirname: &str) -> bool {
    let dir = format!(".\\{}\\", dirname);
    // Ebin :DD
    std::path::Path::new(&dir).exists() || {
        print!("Directory `{}` not found. Create it [y/n]? ", dir);
        std::io::stdout().flush().unwrap();
        let mut answer = String::new();
        if let Ok(_) = std::io::stdin().read_line(&mut answer) {
            if answer.starts_with("y") {
                std::fs::create_dir(dir).unwrap();
                return true
            }
        }
        false
    }
}

// Remove file extension and directory path from input string
pub fn filename(file: &str) -> &str {
    let isuff = match file.rfind('.') { 
        Some(i) => i, 
        None => file.len(),
    };
    let ipref = match file.rfind('\\') {
        Some(i) => i,
        None => 0,
    };
    // NOTE Cannot(?) crash here at runtime due to variable-byte-length
    // indexing, because the indices are found with the standard library
    // functions.
    // ie. String slices should be used with caution, because it
    // means indexing into variable-byte-length sequences (UTF8)!
    &file[ipref+1..isuff]
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
