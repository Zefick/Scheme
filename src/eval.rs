
use crate::object::*;
use crate::scope::*;
use std::rc::Rc;


/// Converts lists to Vec of references to its elements.
fn list_to_vec(mut obj : &Object) -> Result<Vec<Rc<Object>>, String> {
    let mut result = Vec::<Rc<Object>>::new();
    while !obj.is_nil() {
        match obj {
            Object::Pair(a, b) => {
                result.push(Rc::clone(a));
                obj = b.as_ref();
            },
            _ => return Err(format!("list required, but got {:?}", obj).to_string())
        }
    }
    Ok(result)
}

/// Ensures that given object is a list with length `n`
fn expect_args(args : &Object, func : &str, n : usize) -> Result<Vec<Rc<Object>>, String> {
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
pub fn car(obj : Rc<Object>,  _ : Rc<Scope>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "car", 1).and_then(|vec| {
        check_pair(vec.get(0).unwrap()).map(|x| Rc::clone(x.0))
    })
}

/// Returns a second element of a pair which is all elements after first for lists.
pub fn cdr(obj : Rc<Object>, _ : Rc<Scope>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "cdr", 1).and_then(|vec| {
        check_pair(vec.get(0).unwrap()).map(|x| Rc::clone(x.1))
    })
}

pub fn length(obj : Rc<Object>, _ : Rc<Scope>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "length", 1).and_then(|vec| {
        list_to_vec(vec.get(0).unwrap())
                .map(|x| Rc::new(Object::make_int(x.len() as i32)))
    })
}


fn eval_args(args : &Object, scope : Rc<Scope>) -> Result<Rc<Object>, String> {
    match args {
        Object::Pair(a, b) => {
            eval(Rc::clone(a), Rc::clone(&scope))
                .and_then(move |head| eval_args(b, scope)
                    .and_then(move |tail| Ok(Rc::new(Object::Pair(head, tail)))))
        },
        _ => Ok(Rc::new(Object::Nil))
    }
}

fn quote(args : &Object) -> Result<Rc<Object>, String> {
    match args {
        Object::Pair(a, _) => Ok(Rc::clone(a)),
        _ => Err(format!("illegal argument for 'quote': {}", args))
    }
}

fn fn_if(args : &Object, scope : Rc<Scope>) -> Result<Rc<Object>, String> {
    expect_args(args, "if", 3).and_then(|vec| {
        let mut vec = vec.into_iter();
        eval(vec.next().unwrap(), Rc::clone(&scope)).and_then(|val| {
            if !val.is_true() {
                vec.next();
            }
            eval(vec.next().unwrap(), scope)
        })
    })
}

fn fn_let(args : &Object, scope : Rc<Scope>) -> Result<Rc<Object>, String> {
    list_to_vec(args).and_then(|vec| {
        if vec.len() < 2 {
            Err(format!("'let' need at least 2 arguments, {} found", vec.len()))
        } else {
            Ok(vec)
        }
    }).and_then(|let_args| {
        list_to_vec(let_args.get(0).unwrap()).and_then(|args| {
            let mut bindings = Vec::new();
            for arg in args {
                let result = list_to_vec(arg.as_ref()).and_then(|vec| {
                   return if vec.len() >= 2 {
                       let var = vec.get(0).unwrap().as_ref();
                       match var {
                           Object::Symbol(s) => {
                               eval(Rc::clone(vec.get(1).unwrap()), Rc::clone(&scope))
                                   .map(|obj| bindings.push((s.clone(), obj)))
                           },
                           _ => Err(format!("let: need a symbol for binding name, got {}", var).to_string())
                       }
                   } else {
                       Err(format!("let: need a list length 2 for bindings, got {}", arg).to_string())
                   }
                });
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
            }
            return Ok(bindings);
        }).and_then(|bindings| {
            fn_begin_vec(let_args.into_iter().skip(1),
                         Rc::new(Scope::new(bindings.as_slice(), Some(scope))))
        })
    })
}

fn fn_begin_vec(args : impl ExactSizeIterator<Item=Rc<Object>>, scope : Rc<Scope>) -> Result<Rc<Object>, String> {
    if args.len() == 0 {
        return Err("'begin' need at least 1 argument".to_string());
    }
    let mut result : Option<Rc<Object>> = None;
    for arg in args {
        match eval(arg, Rc::clone(&scope)) {
            Err(e) => return Err(e),
            Ok(val) => result = Some(val)
        }
    }
    return Ok(result.unwrap());
}


pub fn fn_begin(args : Rc<Object>, scope : Rc<Scope>) -> Result<Rc<Object>, String> {
    list_to_vec(args.as_ref())
        .and_then(|args| fn_begin_vec(args.into_iter(), scope))
}


pub fn eval(obj : Rc<Object>, scope : Rc<Scope>) -> Result<Rc<Object>, String> {
    match obj.as_ref() {
        // resolve a symbol
        Object::Symbol(s) => {
            Ok(Rc::clone(scope.get(s).unwrap_or(&obj)))
        },
        // invoke a function
        Object::Pair(func, args) => {
            eval(Rc::clone(func), Rc::clone(&scope)).and_then(|rc| {
                match rc.as_ref() {
                    Object::Symbol(s) => {
                        return if s == "quote" {
                            quote(args.as_ref())
                        } else if s == "if" {
                            fn_if(args.as_ref(), scope)
                        } else if s == "let" {
                            fn_let(args.as_ref(), scope)
                        } else {
                            Err(format!("expected a function, found unbound symbol '{}'", s))
                        }
                    },
                    Object::Function(f) => {
                        eval_args(args, Rc::clone(&scope))
                            .and_then(|args| f(args, scope))
                    },
                    _ => Err(format!("Illegal object used as a function: {:?}", func))
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
        let scope = get_global_scope();
        let obj = parse_expression(expr).unwrap().pop().unwrap();
        assert_eq!(eval(Rc::new(obj), Rc::new(scope)), Ok(Rc::new(expected)));
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

        assert_expr("(let ((x 2)) x)", Object::make_int(2));
        assert_expr("(let ((x car) (y '(1 2 3))) (x y))", Object::make_int(1));

        assert_expr("(begin 1 2 3)", Object::make_int(3));
    }

}
