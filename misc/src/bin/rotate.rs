use terminal_toys::{
    smargs::{self, List},
    smärgs,
};

struct MyInput {
    degrees: usize,
    images: List<std::path::PathBuf>,
}

fn main() {
    let MyInput { degrees, images } = terminal_toys::smärgs!(
        "Rotate a bunch of image files by degrees (divisable by 90)",
        MyInput {
            degrees: (
                "Degrees to rotate clockwise",
                vec!["deg"],
                smargs::Kind::Required
            ),
            images: ("An image file path", vec!["f"], smargs::Kind::List(1)),
        },
    )
    .parse(std::env::args())
    .unwrap();

    for img in images.0 {
        let thingy = utils::open_decode(&img).unwrap();
        match (degrees / 90) % 4 {
            1 => {
                println!("Rotating 90 degrees.");
                thingy.rotate90().save(img);
            }
            2 => {
                println!("Rotating 180 degrees.");
                thingy.rotate180().save(img);
            }
            3 => {
                println!("Rotating 270 degrees.");
                thingy.rotate270().save(img);
            }
            _ => {
                println!("Nothing happened");
            }
        }
    }
}
