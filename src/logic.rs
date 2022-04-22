use crate::errors::EvalErr;
use crate::eval::*;
use crate::math::num_equal;
use crate::object::Object;
use crate::scope::Scope;
use crate::service::*;

use std::rc::Rc;

fn make_boolean(b: bool) -> Rc<Object> {
    Rc::new(Object::Boolean(b))
}

pub fn fn_if(args: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<Rc<Object>, EvalErr> {
    let mut vec = expect_args(args, "if", 3)?.into_iter();
    if !eval(&vec.next().unwrap(), scope)?.is_true() {
        vec.next();
    }
    eval(&vec.next().unwrap(), scope)
}

pub fn cond(cond_list: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<Rc<Object>, EvalErr> {
    if cond_list.is_empty() {
        return Err(EvalErr::CondNeedsClause());
    }
    for condition in cond_list {
        let vec = list_to_vec(condition.as_ref())?;
        if vec.is_empty() {
            return Err(EvalErr::CondEmptyClause());
        }
        let predicate = vec.get(0).unwrap();
        let mut is_true = false;
        if Object::Symbol("else".to_string()) == *predicate.as_ref() {
            is_true = true;
        } else {
            let x = eval(predicate, scope)?;
            if x.is_true() {
                if vec.len() == 1 {
                    return Ok(Rc::clone(&x));
                }
                is_true = true;
            }
        }
        if is_true {
            return fn_begin(vec[1..].iter(), scope);
        }
    }
    Ok(undef())
}

pub fn is_boolean(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    expect_1_arg(args, "boolean?").map(|arg| {
        make_boolean(match arg.as_ref() {
            Object::Boolean(_) => true,
            _ => false,
        })
    })
}

pub fn logic_not(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    Ok(make_boolean(!expect_1_arg(args, "not")?.is_true()))
}

pub fn logic_and(args: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<Rc<Object>, EvalErr> {
    let mut result = make_boolean(true);
    for obj in args {
        let x = eval(&obj, scope)?;
        if x.is_true() {
            result = x;
        } else {
            return Ok(x);
        }
    }
    return Ok(result);
}

pub fn logic_or(args: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<Rc<Object>, EvalErr> {
    for obj in args {
        let x = eval(&obj, scope)?;
        if x.is_true() {
            return Ok(x);
        }
    }
    return Ok(make_boolean(false));
}

fn object_equal(obj1: &Rc<Object>, obj2: &Rc<Object>) -> bool {
    match (obj1.as_ref(), obj2.as_ref()) {
        (Object::Number(x), Object::Number(y)) => num_equal(x, y),
        (Object::Pair(car1, cdr1), Object::Pair(car2, cdr2)) => {
            object_equal(car1, car2) && object_equal(cdr1, cdr2)
        }
        _ => obj1 == obj2,
    }
}

/// The softest of equality functions.
/// Recursively compares the contents of pairs, applying `eqv?` on other objects
pub fn fn_equal(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let (obj1, obj2) = expect_2_args(args, "equal?")?;
    Ok(make_boolean(object_equal(&obj1, &obj2)))
}

/// Consider pairs are the same even if it's the same objects
/// So two distinct lists with the same content still different for `eqv?`
pub fn fn_eqv(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let (obj1, obj2) = expect_2_args(args, "eqv?")?;
    let result = match (obj1.as_ref(), obj2.as_ref()) {
        (Object::Pair(..), Object::Pair(..)) => std::ptr::eq(obj1.as_ref(), obj2.as_ref()),
        _ => object_equal(&obj1, &obj2),
    };
    return Ok(make_boolean(result));
}

/// The difference between `eq?` and `eqv?` is that `eq?` taking into account the type of numbers
/// and returns `false` if they are not match
/// even if the numbers are the same for `eqv?` (e.g. 1 and 1.0)
pub fn fn_eq(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let (obj1, obj2) = expect_2_args(args, "eq?")?;
    let result = match (obj1.as_ref(), obj2.as_ref()) {
        (Object::Number(n1), Object::Number(n2)) => n1 == n2,
        _ => object_equal(&obj1, &obj2),
    };
    return Ok(make_boolean(result));
}
