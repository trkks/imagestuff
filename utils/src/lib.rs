use std::io::Write;
use std::path;

use terminal_toys::{smargs, spinner::start_spinner};

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

/// Common argument type for imagestuff bins.
pub type CliConfig = (
    Vec<path::PathBuf>,
    usize,
    usize,
    bool,
    String,
    path::PathBuf,
);

pub fn cli_config() -> Result<CliConfig, String> {
    smargs::arguments!(
        "Convert a picture into 'ASCII' (UTF8)",
        (
            "Path to picture to convert",
            ["source", "s"],
            smargs::Argument::RequiredList
        ),
        (
            "Amount of columns in result (this is doubled for better \
                terminal visuals so give half if you want exact width)",
            ["width", "w"],
            smargs::Argument::Optional("50")
        ),
        (
            "Amount of rows in result",
            ["height", "h"],
            smargs::Argument::Optional("50")
        ),
        (
            "Output 'black on white' for light-colored terminal",
            ["inverted", "i"],
            smargs::Argument::Flag
        ),
        (
            "Palette to use. In case of text rendering, the UTF8 character-set to use",
            ["palette", "p"],
            smargs::Argument::Optional(" -~=o0@#")
        ),
        (
            "Amount of threads to use for rendering",
            ["threads"],
            smargs::Argument::Optional("4")
        ),
        (
            "Filepath to save the result in or '-' for printing to stdout",
            ["out", "o"],
            smargs::Argument::Optional("renders")
        )
    )
    .help_keys(vec!["help"])
    .from_env()
    .map_err(|e| e.to_string())
}
