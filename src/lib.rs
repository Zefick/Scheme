pub mod errors;
pub mod eval;
pub mod object;
pub mod parser;
pub mod scope;

mod functions;
mod lists;
mod logic;
mod math;
mod service;

use std::error::Error;
use std::io::BufRead;
use std::rc::Rc;

use object::Object;
use scope::Scope;

pub fn eval_expr(expr: &str, scope: &Rc<Scope>) -> Result<Rc<Object>, Box<dyn Error>> {
    let iter = parser::parse_expression(&expr)?.into_iter();
    let mut result = Rc::new(Object::Nil);
    for obj in iter {
        result = eval::eval(&Rc::new(obj), scope)?;
    }
    Ok(result)
}

pub fn eval_file(file: &str, scope: &Rc<Scope>) -> Result<(), Box<dyn Error>> {
    let content = std::fs::read_to_string(file);
    let src = content.map_err(|_| format!("file '{}' cannot be opened", file))?;
    eval_expr(&src, scope)?;
    Ok(())
}

pub fn repl() {
    let scope = Rc::new(Scope::from_global());

    if let Some(err) = eval_file("prelude.scm", &scope).err() {
        println!("Error in 'prelude.scm': {}", err)
    }

    // Read-Eval-Print Loop
    (std::io::stdin().lock().lines())
        .map(|str| match eval_expr(&str.unwrap(), &scope) {
            Ok(ok) => ok.to_string(),
            Err(err) => "Error: ".to_string() + &err.to_string(),
        })
        .for_each(|result| println!("{}", result));
}
