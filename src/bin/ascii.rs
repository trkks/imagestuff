use std::convert::TryFrom;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use image::{Rgb, Pixel};

use terminal_toys::{ProgressBar, Smargs};

use imagestuff::utils;


const DEFAULT_PALETTE: [char; 8] = [' ', '-', '~', '=', 'o', '0', '@', '#'];


pub struct AsciiConfig {
    source: PathBuf,
    width: u32,
    height: u32,
    inverted: bool,
    palette: Vec<char>,
}

impl TryFrom<Smargs> for AsciiConfig {
    type Error = Box<dyn std::error::Error>;

    fn try_from(smargs: Smargs) -> Result<Self, Self::Error> {
        let source   = smargs.gets(&["source",   "s"]);
        let width    = smargs.gets(&["width",    "w"]);
        let height   = smargs.gets(&["height",   "h"]);
        let inverted = smargs.gets::<bool>(&["inverted", "i"]);
        let palette  = smargs.gets::<String>(&["palette",  "p"]);

        let source = source.or(smargs.first())
            .ok()
            .or_else(|| {
                eprintln!("Need a source image file as first argument");
                std::process::exit(1);
            }).unwrap();

        // The width and height are optional.
        let width = width.or_else(
            |e| if e.is_not_found() { Ok(50) } else { Err(e) }
        )?;
        let height = height.or_else(
            |e| if e.is_not_found() { Ok(50) } else { Err(e) }
        )?;

        // TODO Could this be handled by Smargs::gets automatically (condition
        // on type: if T == bool { .. } else { .. } )?
        let inverted = inverted.is_ok();

        let palette = if let Ok(s) = palette {
            s.chars().collect::<Vec<char>>()
        } else {
            DEFAULT_PALETTE.to_vec()
        };

        Ok(AsciiConfig { source, width, height, inverted, palette })
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let AsciiConfig {
        source,
        width,
        height,
        inverted,
        palette,
    } = AsciiConfig::try_from(Smargs::from_env()?)?;

    ascii_image(&source, width, height, inverted, &palette)
}

// Using ascii characters, generate a textfile representation of an image
fn ascii_image(
    srcfile: &dyn AsRef<std::path::Path>,
    w: u32,
    h: u32,
    inverted: bool,
    palette: &[char],
) -> Result<(), Box<dyn std::error::Error>> {
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
        let asciipixel = pixel_to_ascii(pixel, inverted, palette);
        // Push twice so that textfile looks more like an image in an editor
        ascii.push(asciipixel);
        ascii.push(asciipixel);

        progress.title(&format!("{}/{}", i+1, n));
        progress.ulap();
    }

    let newfile = format!(
        "./ascii/{}.txt",
        utils::filename(&srcfile).ok_or("No filename on source")?
    );
    println!("\nSaving to {}", newfile);

    let mut file = File::create(newfile)?;

    file.write_all(ascii.into_iter().collect::<String>().as_bytes())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Get an ascii character that looks like the brightness of the pixel.
/// If `inverted`, the text is black on white background and vice versa.
fn pixel_to_ascii(
    pixel: Rgb<u16>,
    inverted: bool,
    palette: &[char],
) -> char {
    // TODO Somehow normalize the "span" of brightness to the palette so that
    // darker images don't just turn straight to black.
    let brightness = pixel.to_luma().0[0] as f32 / u16::MAX as f32;
    let i = (brightness * (palette.len() - 1) as f32) as usize;

    palette[if inverted { palette.len() - 1 - i } else { i }]
}

#[cfg(test)]
mod tests {
    use image::Rgb;
    use super::pixel_to_ascii;
    use super::DEFAULT_PALETTE;
    #[test]
    fn test_pixel_to_ascii() {
        let n = u16::MAX;
        assert_eq!(
            pixel_to_ascii(Rgb([n, n, n]), false, &DEFAULT_PALETTE),
            '#'
        );
        assert_eq!(
            pixel_to_ascii(Rgb([n / 2, n / 2, n / 2]), false, &DEFAULT_PALETTE),
            '='
        );
        assert_eq!(
            pixel_to_ascii(Rgb([0, 0, 0]), false, &DEFAULT_PALETTE),
            ' '
        );

        assert_eq!(
            pixel_to_ascii(Rgb([n, n, n]), true, &DEFAULT_PALETTE),
            ' '
        );
        assert_eq!(
            pixel_to_ascii(Rgb([n / 2, n / 2, n / 2]), true, &DEFAULT_PALETTE),
            'o'
        );
        assert_eq!(
            pixel_to_ascii(Rgb([0, 0, 0]), true, &DEFAULT_PALETTE),
            '#'
        );
    }
}
