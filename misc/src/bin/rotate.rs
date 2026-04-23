use terminal_toys::smargs;

fn main() {
    let (degrees, images): (usize, Vec<std::path::PathBuf>) = smargs::arguments!(
        "Rotate a bunch of image files by degrees (divisable by 90)",
        (
            "Degrees to rotate clockwise",
            vec!["deg"],
            smargs::Argument::Required
        ),
        (
            "An image file path",
            vec!["f"],
            smargs::Argument::RequiredList
        ),
    )
    .parse(std::env::args())
    .unwrap();

    for img in images {
        let thingy = utils::open_decode(&img).unwrap();
        match (degrees / 90) % 4 {
            1 => {
                println!("Rotating 90 degrees.");
                thingy.rotate90().save(img)
            }
            2 => {
                println!("Rotating 180 degrees.");
                thingy.rotate180().save(img)
            }
            3 => {
                println!("Rotating 270 degrees.");
                thingy.rotate270().save(img)
            }
            _ => {
                println!("Nothing happened");
                Ok(())
            }
        }
        .unwrap()
    }
}
