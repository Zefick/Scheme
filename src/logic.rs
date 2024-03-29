use crate::errors::EvalErr;
use crate::eval::*;
use crate::functions::CallResult;
use crate::math::num_equal;
use crate::object::Object;
use crate::scope::Scope;
use crate::service::*;

use std::rc::Rc;

fn make_boolean(b: bool) -> Rc<Object> {
    Rc::new(Object::Boolean(b))
}

pub fn fn_if(args: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<CallResult, EvalErr> {
    let vec = expect_args(args, "if", 3)?;
    let cond = eval(&vec[0], scope)?.is_true();
    let result = if cond { &vec[1] } else { &vec[2] };
    Ok(CallResult::TailCall(Rc::clone(result), Rc::clone(scope)))
}

pub fn cond(cond_list: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<CallResult, EvalErr> {
    if cond_list.is_empty() {
        return Err(EvalErr::CondNeedsClause());
    }
    for condition in cond_list {
        let vec = list_to_vec(&condition)?;
        if vec.is_empty() {
            return Err(EvalErr::CondEmptyClause());
        }
        let predicate = &vec[0];
        let mut is_true = false;
        if &Object::Symbol("else".to_string()) == predicate.as_ref() {
            is_true = true;
        } else {
            let x = eval(predicate, scope)?;
            if x.is_true() {
                if vec.len() == 1 {
                    return Ok(CallResult::Object(x));
                }
                is_true = true;
            }
        }
        if is_true {
            return fn_begin(&vec[1..], scope);
        }
    }
    Ok(CallResult::Object(undef()))
}

pub fn is_boolean(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    expect_1_arg(args, "boolean?")
        .map(|arg| make_boolean(matches!(arg.as_ref(), Object::Boolean(_))))
}

pub fn logic_not(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    Ok(make_boolean(!expect_1_arg(args, "not")?.is_true()))
}

pub fn logic_and(args: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<CallResult, EvalErr> {
    for item in args.iter() {
        if !eval(item, scope)?.is_true() {
            return Ok(CallResult::Object(make_boolean(false)));
        }
    }
    let result = if args.is_empty() {
        make_boolean(true)
    } else {
        args[args.len() - 1].clone()
    };
    // last expression in tail position
    Ok(CallResult::TailCall(result, scope.clone()))
}

pub fn logic_or(args: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<CallResult, EvalErr> {
    for item in args.iter() {
        let result = eval(item, scope)?;
        if result.is_true() {
            return Ok(CallResult::Object(result));
        }
    }
    let result = if args.is_empty() {
        make_boolean(false)
    } else {
        args.last().unwrap().clone()
    };
    Ok(CallResult::TailCall(result, scope.clone()))
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
    Ok(make_boolean(result))
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
    Ok(make_boolean(result))
}
