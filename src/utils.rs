use std::io::{Write, Stdout, Result as IOResult};
use std::iter;
use image::io::{Reader as ImageReader};
use image::{Rgb, DynamicImage};


/// An ascii-graphics bar to easily print out the amount of progress a program
/// has done per iteration of a loop or between some other equal sized tasks.
///
/// # Example:
/// ```
/// let mut progress = ProgessBar::new(1000, 15);
/// progress.set_title("Amount of work done");
/// for _ in range(0..1000) {
///     // -- do work --
///
///     progress.print_update()?; 
///     // Example of output when looped 400 to 499 times:
///     // Amount of work done: [====......]
/// }
/// ```
pub struct ProgressBar {
    source_progress: usize,
    threshold: usize,
    bar: Vec<u8>,
    title: String,
    out: Stdout,
}
impl ProgressBar {
    /// Creates a new ProgressBar of `bar_len` characters wide that fills up
    /// based on update count relative to `source_len`
    pub fn new(source_len: usize, bar_len: usize) -> Self {
        // A bar's length is the length of content not '\r', '[' or ']'
        let bar : Vec<u8> = iter::repeat('.' as u8).take(bar_len)
                                                    .collect();
        ProgressBar {
            source_progress: 0,
            threshold: source_len / bar_len,
            bar,
            title: String::from("Progress"),
            out: std::io::stdout(),
        }
    }

    /// Signal that progress has been made. If this update leads to filling up
    /// the bar, it is printed to stdout for reflecting this new state
    pub fn print_update(&mut self) -> IOResult<()> {
        if self.update() {
            self.print()?
        }
        Ok(())
    }

    /// Change the title shown next to the progress bar
    pub fn title(&mut self, title: &str) {
        self.title = String::from(title);
    }

    fn update(&mut self) -> bool{
        // Source has implied a state update, so increment progress
        self.source_progress += 1;
        if self.source_progress % self.threshold == 0 {
            let index = self.bar.iter().position(|x| *x == '.' as u8)
                        .unwrap_or(0);
            self.bar[index] = '=' as u8;
            return true
        }
        return false
    }

    fn print(&mut self) -> IOResult<()> {
        let prefix = format!("\r{} [", self.title);
        self.out.write(prefix.as_bytes()).map(|_| ())?;
        self.out.write(self.bar.as_slice()).map(|_| ())?;
        let progressed = self.bar.iter().rposition(|x| *x == '=' as u8)
                                        .unwrap_or(0) + 1;
        if progressed < self.bar.len() {
            self.out.write(b"]").map(|_| ())?;
        } else {
            self.out.write(b"] Done!").map(|_| ())?;
        }
        self.out.flush()
    }
}

// Utility for loading an image from filepath into a DynamicImage
pub fn open_decode(file: &str) -> Result<DynamicImage, String> {
    println!("Loading image {}", &file);
    match ImageReader::open(&file) {
        Ok(reader) => reader.decode().map_err(|e| e.to_string()), 
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
