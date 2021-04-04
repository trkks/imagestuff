use std::io::Write;
use std::env;
use image::Rgb as Rgb;
use crate::utils;

type ImgBuffer16 = image::ImageBuffer<Rgb<u16>, Vec<u16>>;


pub struct LerpConfig {
    filepaths: Vec<String>,
}

impl LerpConfig {
    pub fn new(args: env::Args) -> Result<Self, String> {
        let filepaths: Vec<String> = args.collect();
        if filepaths.len() < 2 {
            return Err(
                String::from("Didn't get enough input files (need atleast 2)"))
        }

        Ok(LerpConfig { filepaths })
    }
}

pub fn run(args: env::Args) -> Result<(), String> {
    let config = LerpConfig::new(args).map_err(ToString::to_string)?;

    // TODO Lerp based on the size of first input ie. squeeze/stretch to fit

    // zip the filepaths so, that lerp is being done between every two
    // consecutive paths eg.
    // f1,f2,f3,f4 -> f1:f2, f2:f3, f3:f4
    // f1,f2,f3    -> f1:f2, f2:f3

    // eg. Original = f1,f2,f3,f4
    // ->
    // f1:f2, f2:f3, f3:f4, (f4:None)
    let one_path_skipped =  config.filepaths.iter().skip(1);
    let zipped_paths = config.filepaths.iter().zip(one_path_skipped);
    // TODO load the images into memory beforehand
    for (f1, f2) in zipped_paths {
        println!("Lerping between: '{}' and '{}'", f1, f2);
        lerp_images(f1, f2)?;
    }

    Ok(())
}

// Generate an image that's halfway faded between two images
fn lerp_images(file1: &str, file2: &str) -> Result<(), String>  {
    if !utils::confirm_dir("pics") {
        return Err(format!("Directory not found"));
    }

    // Read image from file
    let (img1, img2) = (utils::open_decode(file1)?.into_rgb16(),
                        utils::open_decode(file2)?.into_rgb16());

    if img1.dimensions() != img2.dimensions()  {
        return Err(format!("The images are not the same size"));
    }

    let w = img1.width();
    let h = img1.height();

    println!("Lerping between images");
    // Lerp between corresponding pixels in the two images and save to a third
    let mut new_pixels :Vec<u16> = Vec::new();
    new_pixels.reserve((w * h * 3) as usize); // cast to usize
    
    // Configure a progress bar
    let mut progress = utils::ProgressBar::new((w * h) as usize, 50);
    progress.set_stdout();

    for (&p1, &p2) in img1.pixels().zip(img2.pixels()) {
        // Add new pixel lerped between two in image
        let [r,g,b] = utils::half_lerp(p1, p2);
        new_pixels.push(r); new_pixels.push(g); new_pixels.push(b); 

        // Display the progress bar
        progress.print_update(); 
    }

    let newfile = format!("./pics/lerp_{}_{}.png",
                          utils::filename(file1), utils::filename(file2));
    println!("Saving to {}", &newfile);

    // Save transformed image as a new file
    ImgBuffer16::from_vec(w, h, new_pixels)
        .unwrap() // NOTE I'm guessing `from_vec` rarely panics
        .save(&newfile)
        .expect(format!("Failed at saving file to: '{}'", newfile).as_str());

    Ok(())
}
