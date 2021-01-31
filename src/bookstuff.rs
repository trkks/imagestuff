// Nested path:
// use std::io;
// use std::cmp::Ordering;
// == 
// use std::{io, cmp::Ordering}; 

// Nested with one being a subpath to other
// use std::{io, io::Write};
// == 
// use std::io{self, Write};

// Bring all aka "the glob operator"
// use std::*;


// `foo` is in the same scope as `fn main` -> no `pub` in front
mod foo {
    pub mod bar {
        // `fuu` is in same scope as `fn beep` -> no `pub` in front
        mod fuu {
            pub mod bore {
                // `mod` around wanted `fn` -> needs `super`,
                // but because `mod`s in same scope, `bore2` does't need `pub`
                pub fn up() { super::bore2::doot() }
            }
            mod bore2 {
                pub fn doot() { }
            }
        }
        pub fn beep() { 
            // Same scope (implicit `bar::` in front?)
            fuu::bore::up();
            // " From `bar` call `top` "
            super::top();
        }
    }
    pub fn top() { }
}


pub fn main(mut _args: Vec<String>) {
    println!("Im doin stringstuff:");
    book_ch8();

    // About modules:
    // Same scope (implicit `crate::` in front?)
    foo::top();
    // `bar` and `bar::beep` need to be `pub`
    foo::bar::beep();

    let v = vec![1,2,3];
    v[99]; // Panics here
}

// Testing the language while reading The Book
fn book_ch8() {
    let mut heapstring = String::from("foo");
    let stackstring = "bar";

    // push_str copies all elements of the string slice argument
    // -> "foo"+"bar" are put on heap continuously
    heapstring.push_str(stackstring);
    println!("{}", heapstring);
    println!("{}", stackstring);
}
