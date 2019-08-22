use crate::eval::*;
use crate::object::Object;
use std::rc::Rc;

/// Returns a first element of a list or a pair.
pub fn car(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "car", 1)
        .and_then(|vec| Ok(Rc::clone(check_pair(vec.get(0).unwrap())?.0)))
}

/// Returns a second element of a pair which is all elements after first for lists.
pub fn cdr(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "cdr", 1)
        .and_then(|vec| Ok(Rc::clone(check_pair(vec.get(0).unwrap())?.1)))
}

pub fn cons(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "cons", 2).map(|vec| {
        Rc::new(Object::Pair(
            Rc::clone(vec.get(0).unwrap()),
            Rc::clone(vec.get(1).unwrap()),
        ))
    })
}

pub fn list(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    Ok(obj)
}

pub fn length(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    let vec = expect_args(obj.as_ref(), "length", 1)?;
    let len = list_to_vec(vec.get(0).unwrap())?.len();
    Ok(Rc::new(Object::make_int(len as i32)))
}

pub fn is_pair(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    let vec = expect_args(&obj, "pair?", 1)?;
    Ok(Rc::new(Object::Boolean(
        match vec.get(0).unwrap().as_ref() {
            Object::Pair(..) => true,
            _ => false,
        },
    )))
}

pub fn is_list(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    let vec = expect_args(&obj, "list?", 1)?;
    Ok(Rc::new(Object::Boolean(
        list_to_vec(vec.get(0).unwrap().as_ref()).is_ok(),
    )))
}
