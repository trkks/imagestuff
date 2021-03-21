use std::env;
mod imagestuff;

fn main() {
    imagestuff::main(env::args().skip(1).collect())
}


/* 
   1010 == 0+2+0+8 == 10
>> 1
-> 0101 == 1+0+4+0 == 5
>> 1
-> 0010 == 0+2+0+0 == 2
*/
