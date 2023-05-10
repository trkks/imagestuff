use std::fs::File;
use std::convert::TryFrom;
use std::io::Write;
use std::path::PathBuf;

use image::{Rgb, Pixel};
use terminal_toys::{Smargs, Smarg, SmargsError, smargs, SmargsBreak, SmargKind as Sk, ProgressBar};


const DEFAULT_PALETTE: &str = " -~=o0@#";

pub fn main() {
    if let Err(e) = cli_config()
        .map(|AsciiConfig { source, width, height, inverted, palette }|
            ascii_image(
                &source,
                width,
                height,
                if inverted { palette.chars().rev().collect() } else { palette.chars().collect() },
            )
        )
    {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

/// Config for storing information needed for reading an image made of pixels
/// (not e.g. vectors) and converting its pixels to a scaled version of
/// requested `palette`.
pub struct AsciiConfig {
    source: PathBuf,
    width: u32,
    height: u32,
    inverted: bool,
    palette: String,
}

fn cli_config() -> Result<AsciiConfig, SmargsBreak> {
    smargs!(
        "Convert a picture into 'ASCII' (UTF8)",
        AsciiConfig {
            source:(
                "Path to picture to convert",
                ["source", "s"],
                Sk::Required
            ),
            width:(
                "Amount of columns in result (this is doubled for better \
                terminal visuals so give half if you want exact width)",
                ["width", "w"],
                Sk::Optional("50"),
            ),
            height:(
                "Amount of rows in result",
                ["height", "h"],
                Sk::Optional("50")
            ),
            inverted:(
                "Output 'black on white' for light-colored terminal",
                ["inverted", "i"],
                Sk::Flag
            ),
            palette:(
                "UTF8 character-set to use",
                ["palette", "p"],
                Sk::Optional(DEFAULT_PALETTE)
            )
        }
    )
    .help_keys(vec!["help"])
    .from_env()
}

// Using ascii characters, generate a textfile representation of an image
fn ascii_image(
    srcfile: &dyn AsRef<std::path::Path>,
    w: u32,
    h: u32,
    palette: Vec<char>,
) -> Result<(), Box<dyn std::error::Error>> {
    utils::confirm_dir("ascii")?;

    // First open imagefile, confirming its validity
    let img = utils::open_decode(srcfile)?;
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
        let asciipixel = pixel_to_ascii(pixel, &palette);
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
    palette: &[char],
) -> char {
    // TODO Somehow normalize the "span" of brightness to the palette so that
    // darker images don't just turn straight to black.
    let brightness = pixel.to_luma().0[0] as f32 / u16::MAX as f32;
    let i = (brightness * (palette.len() - 1) as f32) as usize;

    palette[i]
}

#[cfg(test)]
mod tests {
    use image::Rgb;
    use super::pixel_to_ascii;
    use super::DEFAULT_PALETTE;
    #[test]
    fn test_pixel_to_ascii() {
        let n = u16::MAX;
        let palette = DEFAULT_PALETTE.chars().collect::<Vec<char>>();

        assert_eq!(
            pixel_to_ascii(Rgb([n, n, n]), &palette),
            '#'
        );
        assert_eq!(
            pixel_to_ascii(Rgb([n / 2, n / 2, n / 2]), &palette),
            '='
        );
        assert_eq!(
            pixel_to_ascii(Rgb([0, 0, 0]), &palette),
            ' '
        );
        assert_eq!(
            // Inverted palette.
            pixel_to_ascii(Rgb([n / 2, n / 2, n / 2]), &palette.clone().into_iter().rev().collect::<Vec<char>>()),
            'o'
        );
        assert_eq!(
            pixel_to_ascii(Rgb([0, 0, 0]), &palette),
            ' '
        );
    }
}
