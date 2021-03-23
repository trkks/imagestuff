
// NOTE Using `&str` instead of `String` in these functions, because
// https://doc.rust-lang.org/book/ch04-03-slices.html says it's for "more
// experienced Rustacean[s]"

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
