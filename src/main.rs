use std::{env, process};
use imagestuff::{ascii, lerp};

fn main() {
    let mut args = env::args();
    // Skip the executable name
    args.next();

    match args.next() {
        Some(arg) => {
            match arg.to_lowercase().as_str() {
                "ascii" => ascii::run(args),
                "lerp"  => lerp::run(args),
                _       => {
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


/* 
   1010 == 0+2+0+8 == 10
>> 1
-> 0101 == 1+0+4+0 == 5
>> 1
-> 0010 == 0+2+0+0 == 2
*/
