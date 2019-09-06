mod eval;
mod functions;
mod lists;
mod logic;
mod math;
mod object;
mod parser;
mod scope;
mod service;

#[cfg(test)]
mod tests;

use std::io::stdin;
use std::rc::Rc;

use object::Object;

/// Infinite iterator of S-expressions taken from stdin
/// There may be errors in case of ill-formed expressions
fn read_input() -> impl Iterator<Item = Result<Object, String>> {
    fn next() -> Box<dyn Iterator<Item = Result<Object, String>>> {
        let s = &mut String::new();
        stdin().read_line(s).unwrap();
        match parser::parse_expression(s) {
            Ok(vec) => Box::new(vec.into_iter().map(Ok)),
            Err(e) => Box::new(std::iter::once(Err(e.0))),
        }
    };
    std::iter::from_fn(move || Some(next())).flatten()
}

#[allow(unused)]
fn main() {
    let debug = false;
    let scope = scope::get_global_scope();

    // Read-Eval-Print Loop
    read_input()
        .map(|obj| {
            if debug && obj.is_ok() {
                println!("{:?}", obj.as_ref().unwrap());
            }
            obj.and_then(|obj| eval::eval(&Rc::new(obj), &scope))
                .map(|result| result.to_string())
                .unwrap_or_else(|err| format!("Error: {}", err))
        })
        .for_each(|result| println!("{}", result));
}
