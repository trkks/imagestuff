use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

use crate::utils;


pub struct AsciiConfig {
    source_file: PathBuf,
    width: u32,
    height: u32,
}

impl AsciiConfig {
    pub fn new(mut args: env::Args) -> Result<Self, String> {
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

        Ok(AsciiConfig { source_file, width, height })
    }
}

pub fn run(args: env::Args) -> Result<(), String> {
    let config = AsciiConfig::new(args).map_err(|e| e.to_string())?;
    ascii_image(config.source_file.to_str().unwrap(), 
                config.width, config.height)
}

// Using ascii characters, generate a textfile representation of an image
fn ascii_image(srcfile: &str, w: u32, h: u32) -> Result<(), String>{
    if !utils::confirm_dir("ascii") {
        return Err(format!("Directory not found"));
    }    

    // First open imagefile, confirming its validity
    let img = utils::open_decode(&srcfile).map_err(|e| e.to_string())?;
    // Then turn the image into a processable format for the ascii conversion
    let img = img.resize(w, h, image::imageops::FilterType::Triangle)
                 .grayscale()
                 .into_rgb16();

    let mut ascii = Vec::new();
    let n = img.pixels().len();
    ascii.reserve(n * 2 + h as usize);
    let mut progress = utils::ProgressBar::new(n, 25);
    progress.title("Converting to ascii");
    for (i, &pixel) in img.pixels().enumerate() {
        if i % w as usize == 0 && i != 0 {
            ascii.push('\n');
        }
        let asciipixel = utils::pixel_to_ascii(pixel); 
        // Push twice so that textfile looks more like an image in an editor
        ascii.push(asciipixel);
        ascii.push(asciipixel);

        progress.title(&format!("{}/{}", i+1, n));
        progress.print_update().map_err(|e| e.to_string())?; 
    } 

    let newfile = format!("./ascii/{}.txt", utils::filename(srcfile));
    println!("\nSaving to {}", newfile);

    let mut file = File::create(newfile).map_err(|e| e.to_string())?;

    file.write_all(ascii.into_iter().collect::<String>().as_bytes())
        .map_err(|e| e.to_string())
}
