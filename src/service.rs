//! The module contains widely used functions for more
//! comfortable work with Scheme's object from Rust.

use crate::errors::EvalErr;
use crate::object::Object;
use std::rc::Rc;

/// Converts lists to Vec of references to its elements.
pub fn list_to_vec(obj: &Object) -> Result<Vec<Rc<Object>>, EvalErr> {
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
pub fn vec_to_list(vec: Vec<Rc<Object>>) -> Object {
    vec.into_iter()
        .rfold(Object::Nil, |tail, elem| Object::Pair(elem, Rc::new(tail)))
}

/// Ensures that given object is a list with length `n`
pub fn expect_args(vec: Vec<Rc<Object>>, func: &str, n: usize) -> Result<Vec<Rc<Object>>, EvalErr> {
    if vec.len() != n {
        Err(EvalErr::WrongAgrsNum(func.to_string(), n, vec.len()))
    } else {
        Ok(vec)
    }
}

/// Checks that taken object is a list with one element and returns the element or error
pub fn expect_1_arg(vec: Vec<Rc<Object>>, func: &str) -> Result<Rc<Object>, EvalErr> {
    expect_args(vec, func, 1).map(|vec| vec[0].clone())
}

/// Checks that taken object is a list of two elements
/// and returns a tuple of this elements or error
pub fn expect_2_args(
    vec: Vec<Rc<Object>>, func: &str,
) -> Result<(Rc<Object>, Rc<Object>), EvalErr> {
    expect_args(vec, func, 2).map(|vec| (vec[0].clone(), vec[1].clone()))
}

/// Ensures that given object is a pair or returns an Err.
/// Since Rust doesn't support types for enum variants,
/// we are forced to use a tuple for the Ok value.
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
