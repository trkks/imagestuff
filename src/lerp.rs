use std::env;
use image::Rgb as Rgb;
use terminal_toys::ProgressBar;

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
                String::from("Didn't get enough input files (need at least 2)"))
        }

        Ok(LerpConfig { filepaths })
    }
}

pub fn run(args: env::Args) -> Result<(), String> {
    let config = LerpConfig::new(args).map_err(|e| e.to_string())?;

    // TODO Lerp based on the size of first input ie. squeeze/stretch to fit

    // zip the files so, that lerp is being done between every two consecutive
    // images eg.
    // img1,img2,img3,img4 -> img1:img2, img2:img3, img3:img4
    // img1,img2,img3    -> img1:img2, img2:img3

    // eg. Original = img1,img2,img3,img4
    // ->
    // img1:img2, img2:img3, img3:img4, (img4:None)
    let images = load_all(config.filepaths)?;
    let one_img_skipped =  images.iter().skip(1);
    let zipped_imgs = images.iter().zip(one_img_skipped);
    for (img1, img2) in zipped_imgs {
        println!("Lerping between: '{}' and '{}'", img1.path, img2.path);
        lerp_images(img1, img2)?;
    }

    Ok(())
}

struct ImageData { path: String, buffer: ImgBuffer16, }
fn load_all(files: Vec<String>) -> Result<Vec<ImageData>, String> {
    let mut imgs = Vec::with_capacity(files.len());
    for f in files {
        match utils::open_decode(&f) {
            Ok(img) =>
                imgs.push(ImageData { path: f, buffer: img.into_rgb16() }),
            Err(e)    => return Err(e.to_string()),
        }
    }
    Ok(imgs)
}

// Generate an image that's halfway faded between two images
fn lerp_images(img1: &ImageData, img2: &ImageData) -> Result<(), String>  {
    let ImageData { path: file1, buffer: img1 } = img1;
    let ImageData { path: file2, buffer: img2 } = img2;

    if img1.dimensions() != img2.dimensions()  {
        return Err(format!("The images are not the same size"));
    }

    let w = img1.width();
    let h = img1.height();

    // Lerp between corresponding pixels in the two images and save to a third
    let mut new_pixels :Vec<u16> = Vec::new();
    new_pixels.reserve((w * h * 3) as usize); // cast to usize
    
    // Configure a progress bar
    let mut progress = ProgressBar::new((w * h) as usize, 16);

    for (i, (&p1, &p2)) in img1.pixels().zip(img2.pixels()).enumerate() {
        // Add new pixel lerped between two in image
        let [r,g,b] = half_lerp(p1, p2);
        new_pixels.push(r); new_pixels.push(g); new_pixels.push(b); 

        // Display the progress bar
        progress.title(&format!("Lerpstatus ({}/{})", i+1, w*h));
        progress.print_update().map_err(|e| e.to_string())?; 
    }

    utils::confirm_dir("pics")?;
    let newfile = format!("./pics/lerp_{}_{}.png",
                          utils::filename(file1)?, utils::filename(file2)?);
    println!("\nSaving to {}", &newfile);

    // Save transformed image as a new file
    ImgBuffer16::from_vec(w, h, new_pixels)
        .unwrap() // NOTE I'm guessing `from_vec` rarely panics
        .save(&newfile)
        .expect(format!("Failed at saving file to: '{}'", newfile).as_str());

    Ok(())
}

fn half_lerp(p1: Rgb<u16>, p2: Rgb<u16>) -> [u16;3] {
    // Linear interpolation aped from wikipedia
    // Normalize and subtract and divide by 2 and add p1 and return
    [p1[0] + ((p2[0] as f32 - p1[0] as f32) as u16 >> 1),
     p1[1] + ((p2[1] as f32 - p1[1] as f32) as u16 >> 1),
     p1[2] + ((p2[2] as f32 - p1[2] as f32) as u16 >> 1)]
}
