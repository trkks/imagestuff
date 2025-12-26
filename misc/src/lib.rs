pub mod mosaic;

use image::GenericImageView;

pub fn atlas(images: Vec<image::DynamicImage>) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    // The resulting image will be sized by the maximum dimensions times the
    // number of input images
    let (max_width, max_height) = images.iter().fold((0, 0), |acc, img| {
        let (width, height) = (img.width(), img.height());
        let max_width = if width > acc.0 { width } else { acc.0 };
        let max_height = if height > acc.1 { height } else { acc.1 };
        (max_width, max_height)
    });
    let mut result = image::ImageBuffer::new(max_width * images.len() as u32, max_height);

    // TODO organize width and height according to the wanted dimensions of the
    // resulting atlas
    for (i, image) in images.iter().enumerate() {
        image::imageops::replace(&mut result, image, i as u32 * max_width, 0);
    }

    result
}
