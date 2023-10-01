use std::fs::File;
use std::str::FromStr;
use std::io::Write;
use std::path::PathBuf;

use terminal_toys::{smargs, SmargsBreak, SmargsResult, SmargKind as Sk};


const DEFAULT_PALETTE: &str = " -~=o0@#";

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

pub fn main() {
    match cli_config() {
        Ok(CliConfig { source, width, height, inverted, palette, output }) => {

            // First open imagefile, confirming its validity
            let img = utils::open_decode(&source)
                .expect("bad path for source image");

            match misc::mosaic::ascii_image(
                img,
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
