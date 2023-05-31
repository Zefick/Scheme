//! The module contains widely used functions for more
//! comfortable work with Scheme's object from Rust.

use crate::errors::EvalErr;
use crate::object::{List, Object};
use std::rc::Rc;

/// Converts lists to Vec of references to its elements.
pub fn list_to_vec(obj: &Object) -> Result<List, EvalErr> {
    let mut result = Vec::new();
    let mut tail = obj;
    while let Object::Pair(a, b) = tail {
        result.push(Rc::clone(a));
        tail = b;
    }
    if !tail.is_nil() {
        return Err(EvalErr::ListRequired(obj.to_string()));
    }
    Ok(result)
}

/// Converts Vec of references to a list object.
///
/// This function always succeeds.
pub fn vec_to_list(vec: &[Rc<Object>]) -> Object {
    vec.iter().rfold(Object::Nil, |tail, elem| {
        Object::Pair(elem.clone(), Rc::new(tail))
    })
}

/// Ensures that given object is a list with length `n`
pub fn expect_args(args: List, func: &str, n: usize) -> Result<List, EvalErr> {
    if args.len() != n {
        Err(EvalErr::WrongAgrsNum(func.to_string(), n, args.len()))
    } else {
        Ok(args)
    }
}

/// Checks that taken object is a list with one element and returns the element or error
pub fn expect_1_arg(args: List, func: &str) -> Result<Rc<Object>, EvalErr> {
    Ok(expect_args(args, func, 1)?[0].clone())
}

/// Checks that taken object is a list of two elements
/// and returns a tuple of this elements or error
pub fn expect_2_args(args: List, func: &str) -> Result<(Rc<Object>, Rc<Object>), EvalErr> {
    let vec = expect_args(args, func, 2)?;
    Ok((vec[0].clone(), vec[1].clone()))
}

/// Ensures that given object is a pair or returns an Err.
pub fn check_pair(obj: &Object) -> Result<(&Rc<Object>, &Rc<Object>), EvalErr> {
    match obj {
        Object::Pair(x, y) => Ok((x, y)),
        x => Err(EvalErr::PairRequired(x.to_string())),
    }
}

/// The symbol of undefined value.
/// Used in situations when an expression have not an exact result.
/// E.g. for `cond` where all clauses are false.
pub fn undef() -> Rc<Object> {
    Rc::new(Object::Symbol("#<undef>".to_string()))
}
