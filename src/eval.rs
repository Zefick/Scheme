use crate::functions::*;
use crate::logic::*;
use crate::object::*;
use crate::scope::*;

use std::rc::Rc;

/// Converts lists to Vec of references to its elements.
pub fn list_to_vec(obj: &Object) -> Result<Vec<Rc<Object>>, String> {
    let mut result = Vec::new();
    let mut tail = obj;
    while let Object::Pair(a, b) = tail {
        result.push(Rc::clone(a));
        tail = &b;
    }
    if !tail.is_nil() {
        return Err(format!("list required, but got {}", obj).to_string());
    }
    Ok(result)
}

/// Converts Vec of references to a list object.
///
/// This function always succeeds.
pub fn vec_to_list(vec: Vec<Rc<Object>>) -> Object {
    let mut tail = Object::Nil;
    for element in vec.iter().rev() {
        tail = Object::Pair(Rc::clone(element), Rc::new(tail));
    }
    tail
}

/// Ensures that given object is a list with length `n`
pub fn expect_args(args: &Object, func: &str, n: usize) -> Result<Vec<Rc<Object>>, String> {
    let vec = list_to_vec(args)?;
    if vec.len() != n {
        Err(format!(
            "Wrong number or arguments for '{}': {}",
            func,
            vec.len()
        ))
    } else {
        Ok(vec)
    }
}

/// Ensures that given object is a pair or returns an Err.
/// Since Rust doesn't support types for enum variants,
/// we are forced to use a tuple for the Ok value.
pub fn check_pair(obj: &Object) -> Result<(&Rc<Object>, &Rc<Object>), String> {
    match obj {
        Object::Pair(x, y) => Ok((x, y)),
        x => Err(format!("pair required but got {}", x).to_string()),
    }
}

pub fn eval_args(args: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    Ok(Rc::new(match args {
        Object::Pair(a, b) => Object::Pair(eval(&Rc::clone(a), scope)?, eval_args(b, scope)?),
        _ => Object::Nil,
    }))
}

fn quote(args: &Object) -> Result<Rc<Object>, String> {
    expect_args(args, "quote", 1)
        .and_then(|vec| Ok(Rc::clone(vec.get(0).unwrap())))
        .or_else(|_| Err(format!("malformed quote: need 1 argument, got {}", args).to_string()))
}

fn fn_let(args: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    let let_args = list_to_vec(args)?;
    if let_args.len() < 2 {
        return Err(format!(
            "'let' need at least 2 arguments, {} found",
            let_args.len()
        ));
    }
    let args = list_to_vec(let_args.get(0).unwrap())?;
    let mut bindings = Vec::new();
    for arg in args {
        list_to_vec(arg.as_ref()).and_then(|vec| {
            if vec.len() >= 2 {
                let var = vec.get(0).unwrap().as_ref();
                if let Object::Symbol(s) = var {
                    eval(&vec.get(1).unwrap(), scope).map(|obj| bindings.push((s.clone(), obj)))
                } else {
                    Err(format!("let: need a symbol for binding name, got '{}'", var).to_string())
                }
            } else {
                Err(format!("let: need a list length 2 for bindings, got '{}'", arg).to_string())
            }
        })?;
    }
    fn_begin_vec(
        let_args.into_iter().skip(1),
        &Scope::new(bindings.as_slice(), Some(scope)),
    )
}

pub fn fn_begin_vec(
    args: impl Iterator<Item = Rc<Object>>,
    scope: &Rc<Scope>,
) -> Result<Rc<Object>, String> {
    let mut result = undef();
    for arg in args {
        result = eval(&arg, scope)?;
    }
    Ok(result)
}

pub fn fn_begin(args: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    fn_begin_vec(list_to_vec(args)?.into_iter(), scope)
}

pub fn undef() -> Rc<Object> {
    Rc::new(Object::Symbol("#<undef>".to_string()))
}

/// There are two forms of define:
///
/// `(define id expr)` and `(define (id args...) expr...)`
///
/// Both syntax handled by this function. Defined value added to a current scope
pub fn fn_define(args: &Object, scope: &Rc<Scope>) -> Result<(), String> {
    if let Object::Pair(head, expr) = args {
        match head.as_ref() {
            // (define x expr)
            Object::Symbol(s) => {
                if let Object::Pair(value, tail) = expr.as_ref() {
                    if !tail.as_ref().is_nil() {
                        Err("extra argument for 'define'".to_string())
                    } else {
                        eval(value, scope).map(|obj| scope.bind(s, obj))
                    }
                } else {
                    Err(format!("proper list required for 'define': {}", args).to_string())
                }
            }
            // (define (name args) body)
            Object::Pair(name, args) => {
                if let Object::Symbol(s) = name.as_ref() {
                    scope.bind(s, Rc::new(Function::new(s, args, expr, scope)?));
                    Ok(())
                } else {
                    Err(format!("expected symbol as a function name, found '{}'", name).to_string())
                }
            }
            x => Err(format!("proper list or symbol need for 'define', found: {}", x).to_string()),
        }
    } else {
        Err(format!(
            "a list expected for 'define' arguments, found {}",
            args
        ))
    }
}

fn lambda(args: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    check_pair(args)
        .and_then(|(args, body)| Function::new(&"#<lambda>".to_string(), args, body, scope))
        .map(Rc::new)
}

pub fn eval(obj: &Rc<Object>, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    match obj.as_ref() {
        // resolve a symbol
        Object::Symbol(s) => {
            if (s.starts_with("c") && s.ends_with("r"))
                && (s.len() >= 3 && s.len() <= 6)
                && (s[1..s.len() - 1].replace("a", "").replace("d", "")).is_empty()
            {
                Ok(Rc::new(Object::Function(Function::Dynamic(s.clone()))))
            } else {
                (scope.get(s)).ok_or_else(|| format!("unbound variable '{}'", s).to_string())
            }
        }
        // invoke a function
        Object::Pair(func, args) => {
            if let Object::Symbol(s) = func.as_ref() {
                if s == "quote" {
                    return quote(args);
                } else if s == "if" {
                    return fn_if(args, scope);
                } else if s == "let" {
                    return fn_let(args, scope);
                } else if s == "begin" {
                    return fn_begin(args, &Scope::new(&[], Some(scope)));
                } else if s == "define" {
                    return fn_define(args, scope).map(|_| Rc::new(Object::Nil));
                } else if s == "lambda" {
                    return lambda(args, scope);
                } else if s == "and" {
                    return logic_and(args, scope);
                } else if s == "or" {
                    return logic_or(args, scope);
                } else if s == "cond" {
                    return cond(args, scope);
                }
            }
            if let Object::Function(f) = eval(func, scope)?.as_ref() {
                f.call(eval_args(args, scope)?)
            } else {
                Err(format!("Illegal object used as a function: {}", func))
            }
        }
        // other values evaluates to itself
        _ => Ok(Rc::clone(obj)),
    }
}
