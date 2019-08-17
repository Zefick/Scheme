
use crate::object::*;
use crate::scope::*;
use crate::functions::*;
use crate::logic::*;

use std::rc::Rc;
use std::cell::RefCell;


/// Converts lists to Vec of references to its elements.
pub fn list_to_vec(mut obj : &Object) -> Result<Vec<Rc<Object>>, String> {
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
pub fn expect_args(args : &Object, func : &str, n : usize) -> Result<Vec<Rc<Object>>, String> {
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
pub fn check_pair(obj : &Object) -> Result<(&Rc<Object>, &Rc<Object>), String> {
    match obj {
        Object::Pair(x, y) => Ok((x, y)),
        x => Err(format!("pair required but got {:?}", x).to_string())
    }
}

pub fn eval_args(args : &Object, scope : &Rc<RefCell<Scope>>) -> Result<Rc<Object>, String> {
    match args {
        Object::Pair(a, b) => {
            eval(&Rc::clone(a), scope)
                .and_then(|head| eval_args(b, scope)
                    .and_then(|tail| Ok(Rc::new(Object::Pair(head, tail)))))
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

fn fn_let(args : &Object, scope : &Rc<RefCell<Scope>>) -> Result<Rc<Object>, String> {
    list_to_vec(args).and_then(|let_args| {
        if let_args.len() < 2 {
            return Err(format!("'let' need at least 2 arguments, {} found", let_args.len()));
        }
        list_to_vec(let_args.get(0).unwrap()).and_then(|args| {
            let mut bindings = Vec::new();
            for arg in args {
                let result = list_to_vec(arg.as_ref()).and_then(|vec| {
                   return if vec.len() >= 2 {
                       let var = vec.get(0).unwrap().as_ref();
                       match var {
                           Object::Symbol(s) => {
                               eval(&vec.get(1).unwrap(), scope)
                                   .map(|obj| bindings.push((s.clone(), obj)))
                           },
                           _ => Err(format!("let: need a symbol for binding name, got '{}'", var).to_string())
                       }
                   } else {
                       Err(format!("let: need a list length 2 for bindings, got '{}'", arg).to_string())
                   }
                });
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
            }
            fn_begin_vec(let_args.into_iter().skip(1),
                         &Rc::new(RefCell::new(
                             Scope::new(bindings.as_slice(), Some(scope)))))
        })
    })
}

pub fn fn_begin_vec(args : impl Iterator<Item=Rc<Object>>, scope : &Rc<RefCell<Scope>>)
                    -> Result<Rc<Object>, String> {
    let mut result : Option<Rc<Object>> = None;
    for arg in args {
        match eval(&arg, scope) {
            Err(e) => return Err(e),
            Ok(val) => result = Some(val)
        }
    }
    Ok(result.unwrap_or_else(undef))
}

pub fn fn_begin(args : &Object, scope : &Rc<RefCell<Scope>>) -> Result<Rc<Object>, String> {
    list_to_vec(args).and_then(|args| fn_begin_vec(args.into_iter(), scope))
}

pub fn undef() -> Rc<Object> {
    Rc::new(Object::Symbol("#<undef>".to_string()))
}

/// There are two forms of define:
///
/// `(define id expr)` and `(define (id args...) expr...)`
///
/// Both syntax handled by this function. Defined value added to a current scope
pub fn fn_define(args : &Object, scope : &Rc<RefCell<Scope>>) -> Result<(), String> {
    match args {
        Object::Pair(head, expr) => {
            match head.as_ref() {
                // (define x expr)
                Object::Symbol(s) => {
                    match expr.as_ref() {
                        Object::Pair(value, tail) => {
                            if tail.as_ref() != &Object::Nil {
                                Err("extra argument for 'define'".to_string())
                            } else {
                                eval(value, scope).map(
                                    |obj| scope.as_ref().borrow_mut().bind(s.to_string(), obj))
                            }
                        },
                        _ => Err(format!("proper list required for 'define': {}", args).to_string())
                    }
                },
                // (define (name args) body)
                Object::Pair(name, args) => {
                    match name.as_ref() {
                        Object::Symbol(s) => {
                            let name = s.to_string();
                            Function::new(&name, args, expr, &scope).map(
                                |obj| scope.as_ref().borrow_mut().bind(name, Rc::new(obj)))
                        },
                        _ => Err(format!("expected symbol as a function name, found '{}'", name).to_string())
                    }
                },
                x => Err(format!("proper list or symbol need for 'define', found: {}", x).to_string())
            }
        },
        _ => Err(format!("a list expected for 'define' arguments, found {}", args))
    }
}

fn lambda (args : &Object, scope : &Rc<RefCell<Scope>>) -> Result<Rc<Object>, String> {
    check_pair(args).and_then(|(args, body)| {
        Function::new(&"#<lambda>".to_string(), args, body, scope)
    }).map(Rc::new)
}

pub fn eval(obj : &Rc<Object>, scope : &Rc<RefCell<Scope>>) -> Result<Rc<Object>, String> {
    match obj.as_ref() {
        // resolve a symbol
        Object::Symbol(s) => {
            scope.borrow().get(s)
                .ok_or_else(|| format!("unbound variable '{}'", s).to_string())
        },
        // invoke a function
        Object::Pair(func, args) => {
            if let Object::Symbol(s) = func.as_ref() {
                if s == "quote" {
                    return quote(args)
                } else if s == "if" {
                    return fn_if(args, scope)
                } else if s == "let" {
                    return fn_let(args, scope)
                } else if s == "begin" {
                    let new_scope = Rc::new(RefCell::new(Scope::new(&[], Some(scope))));
                    return fn_begin(args, &new_scope)
                } else if s == "define" {
                    return fn_define(args, scope).map(|_| Rc::new(Object::Nil))
                } else if s == "lambda" {
                    return lambda(args, scope)
                } else if s == "and" {
                    return logic_and(args, scope)
                } else if s == "or" {
                    return logic_or(args, scope)
                } else if s == "cond" {
                    return cond(args, scope)
                }
            }
            eval(func, scope).and_then(|rc| {
                if let Object::Function(f) = rc.as_ref() {
                    eval_args(args, scope).and_then(|args| f.call(args))
                } else {
                    Err(format!("Illegal object used as a function: {:?}", func))
                }
            })
        }
        // other values evaluates to itself
        _ => Ok(Rc::clone(obj))
    }
}
