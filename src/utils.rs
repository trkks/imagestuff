use std::io::{Write, Stdout, Result as IOResult};
use std::iter;
use image::io::{Reader as ImageReader};
use image::{Rgb, DynamicImage};


/// An ascii-graphics bar to easily print out the amount of progress a program
/// has done until a certain limit.
/// # Examples:
/// ```
/// let mut progress = ProgessBar::new(1000, 15);
/// progress.set_title("Amount of work done");
/// progress.set_stdout();
/// for _ in range(0..1000) {
///     // -- do work --
///
///     progress.print_update(); 
///     // Example of output when looped 400 to 499 times:
///     // Amount of work done: [====......]
/// }
/// ```
pub struct ProgressBar {
    source_len: usize,
    source_progress: usize,
    bar: Vec<u8>,
    out: Some<std::io::Stdout>,
    title: String, // TODO This could probably be &str pretty easily
}
impl ProgressBar {
    pub fn new(source_len: usize, bar_len: usize) -> Self {
        // A bar's length is the length of content not '\r', '[' or ']'
        let mut bar = iter::repeat('.' as u8).take(bar_len)
                                             .collect::<Vec<u8>>();
        ProgressBar {
            source_len,
            source_progress: 0,
            bar,
        }
    }

    pub fn update(&mut self) {
        if self.source_progress == self.source_len { return }
        // Source has implied a state update, so increment progress
        self.source_progress += 1;
        // Calcutate a relative length for the current amount of progress
        // and if the bar should be updated or not
        // A bar's length is the length of content not '\r', '[' or ']'
        let len = self.bar.iter()
                          .filter(|x| **x == ('=' as u8))
                          .count();
        // Normalized length should match the amount of bar filled until now
        let norm_len = (self.source_progress as f32 / self.source_len as f32 
                        * len as f32) as usize;
        // Update the bar if normalized length has increased from earlier
        if len < norm_len {
            self.bar[len+1] = '=' as u8;
        }
    }
}

impl std::fmt::Display for ProgressBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.bar.iter().map(|x| *x as char)
                                       .collect::<String>())
    }
}
//pub fn print_progress(len: u32, progress: u32, max: u32, stdout: &mut Stdout) 
//-> IOResult<()> {
//    // Alias to add one so that progress bar closes at the end
//    progress + 1;
//    // Calcutate a relative length for the current amount of progress.
//    // Normalize progress to max and fit to len (=terminal width)
//    let bar_len = (progress as f32 / max as f32 * len as f32) as u32;
//    // TODO after string allocation below is fixed, remove this fuglyness
//    if bar_len % (len / 4) != 0 { return Ok(()) }
//    // Make equivalent strings
//    // FIXME This string-making-bullshit slows down like crazy
//    let done = iter::repeat('=').take(bar_len as usize)
//                                .collect::<String>();
//    let left = iter::repeat('.').take((len - bar_len) as usize)
//                                .collect::<String>();
//    // Print progress bar
//    stdout.write(format!("\r[{}{}]", done, left).as_bytes()).map(|_| ())?;
//    if progress == max {
//        stdout.write(" Done!\n".as_bytes()).map(|_| ())?;
//    }
//    stdout.flush()
//}

// Utility for loading an image from filepath into a DynamicImage
pub fn open_decode(file: &str) -> Result<DynamicImage, String> {
    println!("Loading image {}", &file);
    match ImageReader::open(&file) {
        Ok(reader) => reader.decode().map_err(ToString::to_string), 
        Err(msg)   => Err(format!("{}", msg)),
    }
}

// If dirname directory does not exists under current directory, prompt to
// create it. Return True if exists or created
pub fn confirm_dir(dirname: &str) -> bool {
    let dir = format!(".\\{}\\", dirname);
    // Ebin :DD
    std::path::Path::new(&dir).exists() || {
        print!("Directory `{}` not found. Create it [y/n]? ", dir);
        std::io::stdout().flush().unwrap();
        let mut answer = String::new();
        if let Ok(_) = std::io::stdin().read_line(&mut answer) {
            if answer.starts_with("y") {
                std::fs::create_dir(dir).unwrap();
                return true
            }
        }
        false
    }
}

// Remove file extension and directory path from input string
pub fn filename(file: &str) -> &str {
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
pub fn pixel_to_ascii(pixel: Rgb<u16>) -> char {
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

pub fn half_lerp(p1: Rgb<u16>, p2: Rgb<u16>) -> [u16;3] {
    // Linear interpolation aped from wikipedia
    // Normalize and subtract and divide by 2 and add p1 and return 
    [p1[0] + ((p2[0] as f32 - p1[0] as f32) as u16 >> 1),
     p1[1] + ((p2[1] as f32 - p1[1] as f32) as u16 >> 1),
     p1[2] + ((p2[2] as f32 - p1[2] as f32) as u16 >> 1)]
}
