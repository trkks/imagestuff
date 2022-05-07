use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use image::{Rgb, Pixel};
use terminal_toys::ProgressBar;

use imagestuff::utils;


pub struct AsciiConfig {
    source_file: PathBuf,
    width: u32,
    height: u32,
    inverted: bool,
}

impl AsciiConfig {
    pub fn new(mut args: env::Args) -> Result<Self, String> {
        // Skip executable name
        args.next();

        let source_file = match args.next() {
            // Source file's path is saved as OS specific absolute path
            Some(path) => PathBuf::from(&path).canonicalize()
                                              .map_err(|e| e.to_string())?,
            // Return a text description Error if unsufficient args
            None       => return Err(format!("Didn't get a source file")),
        };
        // The width and height are ordered and optional
        let (width, height) = match (args.next(), args.next()) {
            (Some(w), Some(h)) => 
                (w.parse::<u32>().map_err(|e| format!("{}: {}", e, w))?,
                 h.parse::<u32>().map_err(|e| format!("{}: {}", e, h))?),
            _ => (50, 50),
        };

        let inverted = if let Some(s) = args.next() {
            s == "--inverted"
        } else {
            false
        };

        Ok(AsciiConfig { source_file, width, height, inverted})
    }
}

pub fn main() -> Result<(), String> {
    let config = AsciiConfig::new(env::args()).map_err(|e| e.to_string())?;
    ascii_image(
        config.source_file.to_str().unwrap(), 
        config.width,
        config.height,
        config.inverted
    )
}

// Using ascii characters, generate a textfile representation of an image
fn ascii_image(
    srcfile: &str,
    w: u32,
    h: u32,
    inverted: bool
) -> Result<(), String> {
    utils::confirm_dir("ascii")?;

    // First open imagefile, confirming its validity
    let img = utils::open_decode(&srcfile).map_err(|e| e.to_string())?;
    // Then turn the image into a processable format for the ascii conversion
    let img = img.resize(w, h, image::imageops::FilterType::Triangle)
                 .grayscale()
                 .into_rgb16();

    let mut ascii = Vec::new();
    let n = img.pixels().len();
    ascii.reserve(n * 2 + h as usize);
    let mut progress = ProgressBar::new(n, 25);
    progress.title("Converting to ascii");
    for (i, &pixel) in img.pixels().enumerate() {
        if i % w as usize == 0 && i != 0 {
            ascii.push('\n');
        }
        let asciipixel = pixel_to_ascii(pixel, inverted);
        // Push twice so that textfile looks more like an image in an editor
        ascii.push(asciipixel);
        ascii.push(asciipixel);

        progress.title(&format!("{}/{}", i+1, n));
        progress.lap().map_err(|e| e.to_string())?;
    }

    let newfile = format!(
        "./ascii/{}.txt",
        utils::filename(&srcfile)
            .expect(&format!("No filename: {}", srcfile))
    );
    println!("\nSaving to {}", newfile);

    let mut file = File::create(newfile).map_err(|e| e.to_string())?;

    file.write_all(ascii.into_iter().collect::<String>().as_bytes())
        .map_err(|e| e.to_string())
}

/// Get an ascii character that looks like the brightness of the pixel.
/// If not `inverted`, the text is white on dark background and vice versa.
fn pixel_to_ascii(pixel: Rgb<u16>, inverted: bool) -> char {
    const SHADES: [char; 8] = [' ', '-', '~', '=', 'o', '0', '@', '#'];

    let brightness = pixel.to_luma().0[0] as f32 / u16::MAX as f32;
    let i = (brightness * (SHADES.len() - 1) as f32) as usize;

    SHADES[if inverted { SHADES.len() - 1 - i } else { i }]
}

#[cfg(test)]
mod tests {
    use image::Rgb;
    use super::pixel_to_ascii;
    #[test]
    fn test_pixel_to_ascii() {
        let n = u16::MAX;
        assert_eq!(pixel_to_ascii(Rgb([n, n, n]), false),             '#');
        assert_eq!(pixel_to_ascii(Rgb([n / 2, n / 2, n / 2]), false), '=');
        assert_eq!(pixel_to_ascii(Rgb([0, 0, 0]), false),             ' ');

        assert_eq!(pixel_to_ascii(Rgb([n, n, n]), true),             ' ');
        assert_eq!(pixel_to_ascii(Rgb([n / 2, n / 2, n / 2]), true), 'o');
        assert_eq!(pixel_to_ascii(Rgb([0, 0, 0]), true),             '#');
    }
}
