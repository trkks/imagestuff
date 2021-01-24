use std::env;
use std::fs::File;
use std::io::Write;
use image::io::Reader as ImageReader;
use image::Rgb as Rgb;

// TODO So many unwraps... Figure out the Result cascading?
fn main() {
    let mut args = env::args();
    match args.nth(1) {
        Some(arg) => match arg.as_str() {
            "lerp" => {
                lerp_images(args.next().unwrap(),
                            args.next().unwrap());
            },
            "ascii" => {
                let file = args.next().unwrap();
                // TODO defaulting to 0 is confusing
                let dimensions = match (args.next(), args.next()) { 
                    (Some(w), Some(h)) => 
                        Some( (w.parse::<u32>().unwrap(),
                               h.parse::<u32>().unwrap()) ),
                    _ => {
                        println!("No dimensions specified; \
                                  matching the input image.");
                        None
                    }
                };
                ascii_image(file, dimensions);
            },
            _ => println!("No such function"),
        },
        None => println!("Arguments needed"),
    }
}

// Generate an image that's halfway faded between two images
fn lerp_images(file1: String, file2: String) {
    // Read image from file
    let img1 = ImageReader::open(&file1)
                    .unwrap()
                    .decode()
                    .unwrap()
                    .into_rgb16();
    let img2 = ImageReader::open(file2)
                    .unwrap()
                    .decode()
                    .unwrap()
                    .into_rgb16();
    if img1.dimensions() != img2.dimensions()  {
        println!("Error: the images are not the same size");
        return;
    }
    println!("Images loaded");

    let w = img1.width();
    let h = img1.height();

    // Lerp between corresponding pixels in the two images and save to a third
    let mut new_pixels :Vec<u16> = Vec::new();
    new_pixels.reserve((w * h * 3) as usize); // cast to usize
    for (&p1, &p2) in img1.pixels().zip(img2.pixels()) {
        // Add lerps
        let lerped_pixel = half_lerp(p1, p2);
        for val in &lerped_pixel { 
            new_pixels.push(*val); 
        }
    }

    let newfile = format!("./pics/lerped_{}.png", filename(file1));
    println!("Transform done\nSaving to {}", newfile);

    // Save transformed image as a new file
    image::ImageBuffer::<Rgb<u16>,Vec<u16>>::from_vec(w, h, new_pixels)
        .unwrap() // I suppose from_vec() rarely fails
        .save(newfile)
        .unwrap();
}

// Using ascii characters, generate a textfile representation of an image
// NOTE big image produces a HUGE textfile if no dimensions are specified
fn ascii_image(srcfile: String, dimensions: Option<(u32,u32)>) {
    // Open the imagefile
    let reader = ImageReader::open(&srcfile)
                    .or_else(|msg| {
                        Err(format!("Error opening '{}': {}", srcfile, msg))
                    });
    if reader.is_err() { 
        return; 
    }


    //Turn the image into a processable format
    let img = reader.unwrap()
                    .decode()
                    .unwrap();
    // Resize the image if dimensions were given
    dimensions.or_else(|(w,h)| {
        img.resize(w, h, image::imageops::FilterType::Triangle);
    });
    let img = img.grayscale()
                 .into_rgb16();

    println!("Image loaded");


    let mut ascii = Vec::new();
    let n = img.pixels().len() as u32;
    let (w, h) = (img.width(), img.height());
    ascii.reserve((n * 2 + h) as usize);
    for (i, &pixel) in (0..n).zip(img.pixels())  {
        if i % w == 0 && i != 0 {
            ascii.push('\n');
        }
        let asciipixel = pixel_to_ascii(pixel); 
        // Push twice so that textfile looks more like an image in an editor
        ascii.push(asciipixel);
        ascii.push(asciipixel);
    } 

    let newfile = format!("./ascii/{}.txt", filename(srcfile));
    println!("Saving to {}", newfile);

    let mut file = File::create(newfile).unwrap();
    file.write_all(ascii.into_iter().collect::<String>().as_bytes())
        .unwrap();
}

// Remove file extension and directory path from input string
fn filename(file: String) -> String {
    // First, move `file` to being owned by this function
    //let file = file.to_string();
    match file.as_str().rfind('.') {
        Some(isuff) => match file[..isuff].rfind('\\') {
            Some (ipref) => file[..isuff][ipref+1..].to_string(),
            None => file,
        }
        None => file,
    }
}

// Get ascii character that looks like the brightness of the pixel
fn pixel_to_ascii(pixel: Rgb<u16>) -> char {
    // Divide by more (0.2) than count (3) to make slightly darker
    let brightness = (pixel[0] as f32 / u16::MAX as f32 +
                      pixel[1] as f32 / u16::MAX as f32 +
                      pixel[2] as f32 / u16::MAX as f32) / 3.2;

    if 0.875 <= brightness { return '#'; }
    if 0.75  <= brightness { return '@'; }
    if 0.625 <= brightness { return '0'; }
    if 0.5   <= brightness { return 'o'; }
    if 0.375 <= brightness { return '='; }
    if 0.25  <= brightness { return '~'; }
    if 0.125 <= brightness { return '-'; }    
                             return ' ';
}

fn half_lerp(p1: Rgb<u16>, p2: Rgb<u16>) -> [u16;3] {
    // Linear interpolation aped from wikipedia
    [p1[0] + ((p2[0] as f32 - p1[0] as f32) as u16 >> 1),
     p1[1] + ((p2[1] as f32 - p1[1] as f32) as u16 >> 1),
     p1[2] + ((p2[2] as f32 - p1[2] as f32) as u16 >> 1)]
}
