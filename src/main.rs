use std::env;
mod imagestuff;
mod bookstuff;

fn main() {
    if let Some(arg) = env::args().nth(1) {
        match arg.as_str() {
            "image" => imagestuff::main(env::args().skip(2)
                                                   .collect()),
            "books" => bookstuff::main(env::args().skip(2)
                                                  .collect()),
            _ => println!("No such module!"),
        }
    } else {
        println!("Please provide a module name as 1st argument")
    }
}
