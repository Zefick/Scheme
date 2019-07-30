
mod parser;
mod object;

use std::io::*;

fn main() {
    loop {
        let mut s = String::new();
        stdin().read_line(&mut s).unwrap();
        parser::debug_expression(&s);
    }
}
