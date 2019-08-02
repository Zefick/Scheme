
mod parser;
mod object;
mod eval;

use std::io::*;
use std::rc::Rc;

fn main() {
    loop {
        let mut s = String::new();
        print!(" > ");
        stdout().flush();
        stdin().read_line(&mut s).unwrap();
        parser::debug_expression(&s);

        let _ = match parser::parse_expression(&s) {
            Ok(list) => {
                let scope = &mut eval::get_global_scope();
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
