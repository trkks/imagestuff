use std::env;
use image::Rgb as Rgb;
use crate::utils;


type ImgBuffer16 = image::ImageBuffer<Rgb<u16>, Vec<u16>>;

pub fn run(mut args: env::Args) -> Result<(),String> {
    let file = args.next().ok_or("Did not receive source image")?;

    // Generate an image with every other pixel's color inverted:

    // Check/create output directory
    utils::confirm_dir("pics")?;

    // Read image from file
    let img = utils::open_decode(&file)?.into_rgb16();

    let w = img.width();
    let h = img.height();

    println!("Cursing the image");
    let mut new_pixels :Vec<u16> = Vec::new();
    // Multiply by 3 because each pixel is made up of 3 values (RGB)
    let n = (w * h * 3) as usize;
    new_pixels.reserve(n); // cast to usize
    
    // Configure a progress bar
    let mut progress = utils::ProgressBar::new((w*h) as usize, 16);

    for (i, &pixel) in img.pixels().enumerate() {
        if i % 2 == 0 {
            // Bitwise negation
            let [r,g,b] = [!pixel[0], !pixel[1], !pixel[2]];
            new_pixels.push(r); new_pixels.push(g); new_pixels.push(b); 

        } else {
            new_pixels.push(pixel[0]); 
            new_pixels.push(pixel[1]);
            new_pixels.push(pixel[2]);
        }

        // Display the progress bar
        progress.title(&format!("Curse status ({}/{})", i+1, w*h));
        progress.print_update().map_err(|e| e.to_string())?; 
    }

    let newfile = format!("./pics/cursed_{}.png", utils::filename(&file)?);
    println!("\nSaving to {}", &newfile);

    // Save transformed image as a new file
    ImgBuffer16::from_vec(w, h, new_pixels)
        .unwrap() // NOTE I'm guessing `from_vec` rarely panics
        .save(&newfile)
        .expect(format!("Failed at saving file to: '{}'", newfile).as_str());
    
    Ok(())

}
