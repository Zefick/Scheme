use std::error::Error;
use std::io::BufRead;
use std::rc::Rc;

use object::Object;
use scope::Scope;

mod errors;
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

fn eval_expr(expr: String, scope: &Rc<Scope>) -> Result<Rc<Object>, Box<dyn Error>> {
    let mut iter = parser::parse_expression(&expr)?.into_iter();
    let mut result = Rc::new(Object::Nil);
    while let Some(obj) = iter.next() {
        result = eval::eval(&Rc::new(obj), scope)?;
    }
    Ok(result)
}

fn eval_file(file: &str, scope: &Rc<Scope>) -> Result<(), Box<dyn Error>> {
    let src = std::fs::read_to_string(file)
        .map_err(|_| format!("file '{}' cannot be opened", file).to_string())?;
    eval_expr(src, scope)?;
    Ok(())
}

fn main() {
    let scope = scope::get_global_scope();

    eval_file("prelude.scm", &scope)
        .err()
        .map(|err| println!("Error in 'prelude.scm': {}", err));

    // Read-Eval-Print Loop
    (std::io::stdin().lock().lines())
        .map(|str| match eval_expr(str.unwrap(), &scope) {
            Ok(ok) => ok.to_string(),
            Err(err) => "Error: ".to_string() + &err.to_string(),
        })
        .for_each(|result| println!("{}", result));
}
