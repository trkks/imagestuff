use std::fs::File;
use std::str::FromStr;
use std::io::Write;
use std::path::PathBuf;

use image::{Rgb, Pixel};
use terminal_toys::{smargs, SmargsBreak, SmargsResult, SmargKind as Sk, ProgressBar};


const DEFAULT_PALETTE: &str = " -~=o0@#";

pub fn main() {
    match cli_config() {
        Ok(CliConfig { source, width, height, inverted, palette, output }) => {
            match ascii_image(
                &source,
                width,
                height,
                if inverted { palette.chars().rev().collect() } else { palette.chars().collect() },
            ) {
                Ok(ascii) => {
                    match output.0 {
                        Ok(StdoutOrPath::Path(p)) => {
                            let mut output = PathBuf::new();
                            if p.is_dir() {
                                output.push(p);
                                output.push(&source.file_stem().expect("source file bad"));
                                output.set_extension("txt");
                            } else if let Some(parent) = p.parent() {
                                if let Some(dir) = parent.file_name() {
                                    if let Err(e) = utils::confirm_dir(&dir) {
                                        eprintln!("{}", e);
                                        std::process::exit(1);
                                    }
                                }
                                output.push(p)
                            } else {
                                output.push(&source.file_stem().expect("source file bad"));
                                output.set_extension("txt");
                            }
                            eprintln!("\nSaving to {}", output.display());

                            let mut file = match File::create(output) {
                                Ok(x) => x,
                                Err(e) => {
                                    eprintln!("{}", e);
                                    std::process::exit(1);
                                }
                            };

                            if let Err(e) = file.write_all(
                                ascii.as_bytes()
                            ) {
                                eprintln!("{}", e);
                                std::process::exit(1);
                            }
                        },
                        Ok(StdoutOrPath::Stdout) => {
                            println!("{}", ascii)
                        },
                        Err(e) => {
                            eprintln!("{}", e);
                            std::process::exit(1);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
    }
}

#[derive(Debug)]
enum StdoutOrPath {
    Path(PathBuf),
    Stdout,
}

impl FromStr for StdoutOrPath {
    type Err = <PathBuf as FromStr>::Err;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "-" {
            Ok(Self::Stdout)
        } else {
            Ok(Self::Path(PathBuf::from_str(value)?))
        }
    }
}

/// Config for storing information needed for reading an image made of pixels
/// (not e.g. vectors) and converting its pixels to a scaled version of
/// requested `palette`.
#[derive(Debug)]
struct CliConfig {
    source: PathBuf,
    width: u32,
    height: u32,
    inverted: bool,
    palette: String,
    output: SmargsResult<StdoutOrPath>,
}

fn cli_config() -> Result<CliConfig, SmargsBreak> {
    smargs!(
        "Convert a picture into 'ASCII' (UTF8)",
        CliConfig {
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
            ),
            output:(
                "Filepath to save the result in or '-' for printing to stdout",
                ["out", "o"],
                Sk::Maybe
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
) -> Result<String, Box<dyn std::error::Error>> {
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
        //progress.ulap();
    }

    Ok(ascii.into_iter().collect::<String>())
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
