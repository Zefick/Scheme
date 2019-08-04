
use super::object::*;
use std::collections::HashMap;
use std::rc::Rc;


/// Converts lists to Vec of references to its elements.
fn list_to_vec(mut obj : &Object) -> Result<Vec<&Rc<Object>>, String> {
    let mut result = Vec::<&Rc<Object>>::new();
    while !obj.is_nil() {
        match obj {
            Object::Pair(a, b) => {
                result.push(a);
                obj = b.as_ref();
            },
            _ => return Err(format!("list required, but got {:?}", obj).to_string())
        }
    }
    Ok(result)
}

/// Ensures that given object is a list with length `n`
fn expect_args<'a, 'b>(args : &'a Object, func : &'b str, n : usize) -> Result<Vec<&'a Rc<Object>>, String> {
    list_to_vec(args).and_then(|vec| {
        if vec.len() != n {
            Err(format!("Wrong number or arguments for '{}': {}", func, vec.len()))
        } else {
            Ok(vec)
        }
    })
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
    expect_args(obj.as_ref(), "car", 1).and_then(|vec| {
        check_pair(vec.get(0).unwrap()).map(|x| Rc::clone(&x.0))
    })
}

/// Returns a second element of a pair which is all elements after first for lists.
fn cdr(obj : Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "cdr", 1).and_then(|vec| {
        check_pair(vec.get(0).unwrap()).map(|x| Rc::clone(&x.1))
    })
}

fn length(obj : Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "length", 1).and_then(|vec| {
        list_to_vec(vec.get(0).unwrap())
                .map(|x| Rc::new(Object::make_int(x.len() as i32)))
    })
}


pub fn get_global_scope() -> HashMap<String, Rc<Object>> {
    let mut scope = HashMap::<String, Rc<Object>>::new();
    scope.insert("#t".to_string(), Rc::new(Object::Boolean(true)));
    scope.insert("#f".to_string(), Rc::new(Object::Boolean(false)));
    scope.insert("car".to_string(), Rc::new(Object::Function(car)));
    scope.insert("cdr".to_string(), Rc::new(Object::Function(cdr)));
    scope.insert("length".to_string(), Rc::new(Object::Function(length)));
    return scope;
}

fn eval_args(args : &Object, scope : &HashMap<String, Rc<Object>>) -> Result<Rc<Object>, String> {
    match args {
        Object::Pair(a, b) => {
            return eval(Rc::clone(a), scope)
                .and_then(move |head| eval_args(b, scope)
                    .and_then(move |tail| Ok(Rc::new(Object::Pair(head, tail)))))
        },
        _ => Ok(Rc::new(Object::Nil))
    }
}

fn quote(args : &Object) -> Result<Rc<Object>, String> {
    match args {
        Object::Pair(a, _) => {
            Ok(Rc::clone(a))
        },
        _ => Err(format!("illegal argument for 'quote': {}", args))
    }
}

fn fn_if(args : &Object, scope : &HashMap<String, Rc<Object>>) -> Result<Rc<Object>, String> {
    expect_args(args, "if", 3).and_then(|vec| {
        let mut vec = vec.into_iter();
        eval(Rc::clone(vec.next().unwrap()), scope).and_then(|val| {
            if !val.is_true() {
                vec.next();
            }
            eval(Rc::clone(vec.next().unwrap()), scope)
        })
    })
}

pub fn eval(obj : Rc<Object>, scope : &HashMap<String, Rc<Object>>) -> Result<Rc<Object>, String> {
    match obj.as_ref() {
        // resolve a symbol
        Object::Symbol(s) => {
            return Ok(Rc::clone(scope.get(s).unwrap_or(&obj)));
        },
        // invoke a function
        Object::Pair(func, args) => {
            eval(Rc::clone(func), scope).and_then(|rc| {
                match rc.as_ref() {
                    Object::Symbol(s) => {
                        if s == "quote" {
                            return quote(args.as_ref());
                        } else if s == "if" {
                            return fn_if(args.as_ref(), scope);
                        }
                        Err(format!("Unbound symbol: {}", s))
                    },
                    Object::Function(f) => {
                        eval_args(args, scope).and_then(f)
                    },
                    _ => Err(format!("Illegal object used as a function: {}", func))
                }
            })
        }
        // other values evaluates to itself
        _ => Ok(obj)
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use crate::parser::parse_expression;

    fn assert_expr(expr : &str, expected : Object) {
        let scope = &get_global_scope();
        let obj = parse_expression(expr).unwrap().pop().unwrap();
        assert_eq!(eval(Rc::new(obj), scope), Ok(Rc::new(expected)));
    }

    #[test]
    fn eval_test() {
        assert_expr("'1", Object::make_int(1));
        assert_expr("(car '(1 . 2))", Object::make_int(1));
        assert_expr("(cdr '(1 . 2))", Object::make_int(2));
        assert_expr("(car (cdr '(1 2 3)))", Object::make_int(2));
        assert_expr("(length '(1 2 3))", Object::make_int(3));

        assert_expr("(if #t 1 2)", Object::make_int(1));
        assert_expr("(if #f 1 2)", Object::make_int(2));
    }

}
