use std::fs::File;
use std::io::Write;
use image::io::Reader as ImageReader;
use image::Rgb as Rgb;

type ImgBuffer16 = image::ImageBuffer<Rgb<u16>, Vec<u16>>;

// NOTE Using `&str` instead of `String` in these functions, because
// https://doc.rust-lang.org/book/ch04-03-slices.html says it's for "more
// experienced Rustacean[s]"

// TODO So many unwraps... Figure out the Result cascading?
pub fn main(args: Vec<String>) {
    let arg = args.get(0).expect("Need to select a function from the module");

    // Choose between the functions
    match arg.as_str() {
        "lerp" => {
            lerp_images(args.get(1).expect("First file missing"),
                        args.get(2).expect("Second file missing"));
                        //.or_else(|err| panic!("Failed to lerp: {:?}", err));
        },
        "ascii" => {
            let file = args.get(1)
                           .expect("An input file must be specified!");
            let (w,h) = (args.get(2).expect("Width must be specified!"),
                         args.get(3).expect("Height must be specified!"));
            // Parse the dimensions from strings
            let (w,h) = (w.parse::<u32>().expect("Cannot parse width."),
                         h.parse::<u32>().expect("Cannot parse height."));

            ascii_image(&file[..], w, h);
        },
        _ => println!("No such function"),
    }
}


// Generate an image that's halfway faded between two images
fn lerp_images(file1: &str, file2: &str) {
    if !confirm_dir("pics") {
        panic!("Directory not found");
    }

    // Read image from file
    let (img1, img2) = (open_decode(file1).into_rgb16(),
                        open_decode(file2).into_rgb16());

    if img1.dimensions() != img2.dimensions()  {
        panic!("The images are not the same size");
    }

    let w = img1.width();
    let h = img1.height();

    println!("Lerping between images");
    // Lerp between corresponding pixels in the two images and save to a third
    let mut new_pixels :Vec<u16> = Vec::new();
    new_pixels.reserve((w * h * 3) as usize); // cast to usize
    for (&p1, &p2) in img1.pixels().zip(img2.pixels()) {
        // Add lerps
        let lerped_pixel = half_lerp(p1, p2);
        for &val in lerped_pixel.iter() { 
            new_pixels.push(val); 
        }
    }

    let newfile = format!("./pics/lerped_{}.png", filename(file1));
    println!("Saving to {}", &newfile);

    // Save transformed image as a new file
    ImgBuffer16::from_vec(w, h, new_pixels)
        .unwrap() // NOTE I'm guessing `from_vec` rarely panics
        .save(&newfile)
        .expect(format!("Failed at saving file to: '{}'", newfile).as_str());
}

// Using ascii characters, generate a textfile representation of an image
fn ascii_image(srcfile: &str, w: u32, h: u32) {
    if !confirm_dir("ascii") {
        println!("Directory not found, exiting.");
        return;
    }    

    // Open imagefile and turn the image into a processable format
    let img = open_decode(&srcfile)
                  .resize(w, h, image::imageops::FilterType::Triangle)
                  .grayscale()
                  .into_rgb16();

    println!("Converting to ascii");
    let mut ascii = Vec::new();
    let n = img.pixels().len() as u32;
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

// Utility for loading an image from filepath into a DynamicImage
fn open_decode(file: &str) -> image::DynamicImage {
    println!("Loading image {}", &file);
    ImageReader::open(&file)
        .expect(format!("Failed to open file '{}'", &file).as_str())
        .decode()
        .expect(format!("Failed to decode image '{}'", file).as_str())
}

// If dirname directory does not exists under current directory, prompt to
// create it. Return True if exists or created
fn confirm_dir(dirname: &str) -> bool {
    let dir = format!(".\\{}\\", dirname);
    println!("Confirming '{}' exists", dir);
    std::path::Path::new(&dir).exists() || {
        print!("Directory `{}` not found. Create it [y/n]? ", dir);
        std::io::stdout().flush().unwrap();
        let mut answer = String::new();
        match std::io::stdin().read_line(&mut answer) {
            Ok(_) => {
                if answer.as_str().contains('y') {
                    std::fs::create_dir(dir).unwrap();
                    true
                } else {
                    false
                }
            }, 
            _ => false
        }
    }
}

// Remove file extension and directory path from input string
fn filename(file: &str) -> &str {
    let isuff = match file.rfind('.') { 
        Some(i) => i, 
        None => file.len(),
    };
    let ipref = match file.rfind('\\') {
        Some(i) => i,
        None => 0,
    };
    // NOTE Cannot(?) crash here at runtime due to variable-byte-length
    // indexing, because the indices are found with the standard library
    // functions.
    // ie. String slices should be used with caution, because it
    // means indexing into variable-byte-length sequences (UTF8)!
    &file[ipref+1..isuff]
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
