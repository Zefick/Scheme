
mod parser;
mod object;
mod eval;
mod scope;

use std::io::*;
use std::rc::Rc;

#[allow(unused)]

fn main() {
    let debug = false;
    loop {
        let mut s = String::new();
        print!(" > ");
        stdout().flush();
        stdin().read_line(&mut s).unwrap();
        if debug {
            parser::debug_expression(&s);
        }
        match parser::parse_expression(&s) {
            Ok(list) => {
                let scope = &scope::get_global_scope();
                for obj in list {
                    match eval::eval(Rc::new(obj), scope) {
                        Ok(x) => print!("{}\n", x),
                        Err(s) => print!("Error: {}\n", s),
                    }
                }
            },
            Err(parser::ParseErr(s)) => {
                print!("{}\n", s);
            }
        };
    }
}
