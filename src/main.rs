use std::{env, process};
use imagestuff;


const MODULES: [(&str, fn(env::Args) -> Result<(), String>);4] = [
    ("ascii"  , imagestuff::ascii::run),
    ("lerp"   , imagestuff::lerp::run),
    ("cursed" , imagestuff::cursed::run),
    ("raycast", imagestuff::raycast::run),
];

fn main() {
    let mut args = env::args();
    // Skip the executable name
    args.next();

    if let Some(arg) = args.next() {
        if let Some((_, function))
            = MODULES.iter().find(|(name, _)| *name == arg.to_lowercase()) {
            function(args)
        } else {
            eprintln!("No such module");
            process::exit(1);
        }
    } else {
        eprintln!("Please select a module");
        process::exit(1);
    }.unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        process::exit(1);
    })
}
