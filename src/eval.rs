
use super::object::*;
use std::collections::HashMap;
use std::rc::Rc;


///Converts lists to Vec of references to its elements.
fn list_to_vec(mut obj : &Object) -> Result<Vec<&Object>, String> {
    let mut result = Vec::<&Object>::new();
    while !obj.is_nil() {
        match obj {
            Object::Pair(a, b) => {
                result.push(a.as_ref());
                obj = b.as_ref();
            },
            _ => return Err(format!("list required, but got {:?}", obj).to_string())
        }
    }
    Ok(result)
}


/// Ensures that given object is a pair or returns an Err.
/// Since Rust doesn't support types for enum variants,
/// we are forced to use a tuple for the Ok value.
fn check_pair(obj : &Object) -> Result<(&Rc<Object>, &Rc<Object>), String> {
    match obj {
        Object::Pair(x, y) => Ok((x, y)),
        x => Err(format!("pair required but got {:?}", x).to_string())
    }
}


/// Returns a first element of a list or a pair.
fn car(obj : Rc<Object>) -> Result<Rc<Object>, String> {
    match list_to_vec(obj.as_ref()) {
        Ok(vec) => {
            if vec.len() != 1 {
                Err(format!("Wrong number or arguments for 'car': {}", vec.len()))
            } else {
                check_pair(vec.get(0).unwrap())
                        .map(|(x, _)| Rc::clone(&x))
            }
        },
        Err(e) => Err(e)
    }
}

/// Returns a second element of a pair which is all elements after first for lists.
fn cdr(obj : Rc<Object>) -> Result<Rc<Object>, String> {
    match list_to_vec(obj.as_ref()) {
        Ok(vec) => {
            if vec.len() != 1 {
                Err(format!("Wrong number or arguments for 'cdr': {}", vec.len()))
            } else {
                check_pair(vec.get(0).unwrap())
                        .map(|(_, x)| Rc::clone(&x))
            }
        },
        Err(e) => Err(e)
    }
}

fn length(obj : Rc<Object>) -> Result<Rc<Object>, String> {
    match list_to_vec(obj.as_ref()) {
        Ok(vec) => {
            if vec.len() != 1 {
                Err(format!("Wrong number or arguments for 'length': {}", vec.len()))
            } else {
                list_to_vec(vec.get(0).unwrap())
                        .map(|x| Rc::new(Object::make_int(x.len() as i32)))
            }
        },
        Err(e) => Err(e)
    }
}


pub fn get_global_scope() -> HashMap<String, Rc<Object>> {
    let mut scope = HashMap::<String, Rc<Object>>::new();
    scope.insert("car".to_string(), Rc::new(Object::Function(car)));
    scope.insert("cdr".to_string(), Rc::new(Object::Function(cdr)));
    scope.insert("length".to_string(), Rc::new(Object::Function(length)));
    return scope;
}

pub fn eval(obj : Rc<Object>, scope : &HashMap<String, Rc<Object>>) -> Result<Rc<Object>, String> {
    match obj.as_ref() {
        // resolve a symbol
        Object::Symbol(s) => {
            return Ok(Rc::clone(scope.get(s).unwrap_or(&obj)));
        },
        // invoke a function
        Object::Pair(func, args) => {
            match eval(Rc::clone(func), scope) {
                Ok(rc) =>
                    match rc.as_ref() {
                        Object::Function(f) => f(Rc::clone(args)),
                        _ => Err(format!("Illegal object used as a function: {:?}", func))
                    }
                ,
                e => e
            }
        }
        // other values evaluates to itself
        _ => Ok(obj)
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use crate::parser::parse_expression;

    #[test]
    fn eval_test() {
        let scope = &get_global_scope();

        let obj = parse_expression("(car (1 . 2))").unwrap().pop().unwrap();
        assert_eq!(eval(Rc::new(obj), scope),
                   Ok(Rc::new(Object::Number(Number::Integer(1)))));

        let obj = parse_expression("(cdr (1 . 2))").unwrap().pop().unwrap();
        assert_eq!(eval(Rc::new(obj), scope),
                   Ok(Rc::new(Object::Number(Number::Integer(2)))));

        let obj = parse_expression("(length (1 2 3))").unwrap().pop().unwrap();
        assert_eq!(eval(Rc::new(obj), scope),
                   Ok(Rc::new(Object::Number(Number::Integer(3)))));
    }

}
