use crate::eval::*;
use crate::object::Object;
use std::rc::Rc;

/// Returns a first element of a list or a pair.
pub fn car(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "car", 1)
        .and_then(|vec| check_pair(vec.get(0).unwrap()).map(|x| Rc::clone(x.0)))
}

/// Returns a second element of a pair which is all elements after first for lists.
pub fn cdr(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "cdr", 1)
        .and_then(|vec| check_pair(vec.get(0).unwrap()).map(|x| Rc::clone(x.1)))
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
    expect_args(obj.as_ref(), "length", 1).and_then(|vec| {
        list_to_vec(vec.get(0).unwrap()).map(|x| Rc::new(Object::make_int(x.len() as i32)))
    })
}

pub fn is_pair(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(&obj, "pair?", 1).and_then(|vec| {
        Ok(Rc::new(Object::Boolean(
            match vec.get(0).unwrap().as_ref() {
                Object::Pair(..) => true,
                _ => false,
            },
        )))
    })
}

pub fn is_list(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(&obj, "list?", 1).and_then(|vec| {
        Ok(Rc::new(Object::Boolean(
            list_to_vec(vec.get(0).unwrap().as_ref()).is_ok(),
        )))
    })
}
