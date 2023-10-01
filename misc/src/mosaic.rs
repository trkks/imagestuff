use image::{Rgb, Pixel, DynamicImage};


// Using ascii characters, generate a textfile representation of an image
pub fn ascii_image(
    img: DynamicImage,
    w: u32,
    h: u32,
    palette: Vec<char>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Then turn the image into a processable format for the ascii conversion
    let img = img.resize(w, h, image::imageops::FilterType::Triangle)
                 .grayscale()
                 .into_rgb16();

    let mut ascii = Vec::new();
    let n = img.pixels().len();
    ascii.reserve(n * 2 + h as usize);
    for (i, &pixel) in img.pixels().enumerate() {
        if i % w as usize == 0 && i != 0 {
            ascii.push('\n');
        }
        let asciipixel = pixel_to_ascii(pixel, &palette);
        // Push twice so that textfile looks more like an image in an editor
        ascii.push(asciipixel);
        ascii.push(asciipixel);
    }

    Ok(ascii.into_iter().collect::<String>())
}

/// Get an ascii character that looks like the brightness of the pixel.
/// If `inverted`, the text is black on white background and vice versa.
fn pixel_to_ascii(
    pixel: Rgb<u16>,
    palette: &[char],
) -> char {
    // TODO Somehow normalize the "span" of brightness to the palette so that
    // darker images don't just turn straight to black.
    let brightness = pixel.to_luma().0[0] as f32 / u16::MAX as f32;
    let i = (brightness * (palette.len() - 1) as f32) as usize;

    palette[i]
}

#[cfg(test)]
mod tests {
    use image::Rgb;
    use super::pixel_to_ascii;
    #[test]
    fn test_pixel_to_ascii() {
        let n = u16::MAX;
        let palette = " -~=o0@#".chars().collect::<Vec<char>>();

        assert_eq!(
            pixel_to_ascii(Rgb([n, n, n]), &palette),
            '#'
        );
        assert_eq!(
            pixel_to_ascii(Rgb([n / 2, n / 2, n / 2]), &palette),
            '='
        );
        assert_eq!(
            pixel_to_ascii(Rgb([0, 0, 0]), &palette),
            ' '
        );
        assert_eq!(
            // Inverted palette.
            pixel_to_ascii(Rgb([n / 2, n / 2, n / 2]), &palette.clone().into_iter().rev().collect::<Vec<char>>()),
            'o'
        );
        assert_eq!(
            pixel_to_ascii(Rgb([0, 0, 0]), &palette),
            ' '
        );
    }
}
