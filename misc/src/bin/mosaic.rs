use std::fs::File;
use std::io::Write;
use std::path;
use std::str::FromStr;

#[derive(Debug)]
enum Palette {
    Ascii(String),
    //RaycastSpheres,
}

impl Palette {
    fn file_extension(&self) -> String {
        match self {
            Self::Ascii(_) => String::from("txt"),
        }
    }
}

impl FromStr for Palette {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            //"spheres" => Self::RaycastSpheres,
            x => Self::Ascii(x.to_owned()),
        })
    }
}

pub fn main() {
    let (sources, width, height, inverted, palette, output_dir) = utils::cli_config().unwrap();

    let palette = palette.parse().expect("bad palette");

    let renderer = match palette {
        Palette::Ascii(ref palette) => move |source_image| {
            misc::mosaic::ascii_image(
                source_image,
                width as u32,
                height as u32,
                if inverted {
                    palette.chars().rev().collect()
                } else {
                    palette.chars().collect()
                },
            )
            .map(|x| x.bytes().collect::<Vec<u8>>())
        },
    };

    for source in sources {
        // First open imagefile, confirming its validity
        let img = utils::open_decode(&source).expect("bad path for source image");
        let result_data = renderer(img).expect("failed to render image");

        if output_dir == path::PathBuf::new() {
            // Empty directory means that the data should just be printed.
            println!("{}", String::from_utf8_lossy(&result_data))
        } else {
            let mut output = path::PathBuf::new();
            if output_dir.is_dir() {
                utils::confirm_dir(&output_dir).expect("bad path for output dir");
                output.push(&output_dir)
            } else if let Some(parent) = output_dir.parent() {
                eprintln!("Truncating output path to parent dir");
                output = parent.to_path_buf();
            }
            output.push(&source.file_stem().expect("source file bad"));
            output.set_extension(palette.file_extension());
            eprintln!("Saving to {}", output.display());

            let mut file = File::create(output).expect("failed to create output path");

            file.write_all(&result_data).unwrap();
        }
    }
}
