extern crate cabl;
use std::io;
use std::io::Read;


fn main() {
    let mut s = String::new();
    {
        io::stdin().read_to_string(&mut s);
    }
    let mut c = cabl::Cabl::new(s.chars());
    c.process();
}
