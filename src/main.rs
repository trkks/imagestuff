use std::{env, process};
use imagestuff;

fn main() {
    let mut args = env::args();
    // Skip the executable name
    args.next();

    match args.next() {
        Some(arg) => {
            match arg.to_lowercase().as_str() {
                "ascii"   => imagestuff::ascii::run(args),
                "lerp"    => imagestuff::lerp::run(args),
                "cursed"  => imagestuff::cursed::run(args),
                "raycast" => imagestuff::raycast::run(args),
                _         => {
                    eprintln!("No such module");
                    process::exit(1);
                },
            }
        },
        None => {
            eprintln!("Please select a module");
            process::exit(1);
        },
    }.unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
    })
}
