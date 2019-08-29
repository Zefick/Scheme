use crate::eval::*;
use crate::object::Object;
use std::rc::Rc;

/// The family of functions which names match with pattern `c[ad]{1,4}r`.
pub fn cadr(name: &str, obj: &Rc<Object>) -> Result<Rc<Object>, String> {
    let mut obj = expect_args(&obj, name, 1)?.pop().unwrap();
    for op in name[1..name.len() - 1].chars().rev() {
        let pair = check_pair(&obj)?;
        obj = Rc::clone(if op == 'a' { pair.0 } else { pair.1 });
    }
    return Ok(obj);
}

pub fn cons(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(&obj, "cons", 2).map(|vec| {
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
    let vec = expect_args(&obj, "length", 1)?;
    let len = list_to_vec(vec.get(0).unwrap())?.len();
    Ok(Rc::new(Object::make_int(len as i32)))
}

pub fn is_pair(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    let vec = expect_args(&obj, "pair?", 1)?;
    Ok(Rc::new(Object::Boolean(
        check_pair(vec.get(0).unwrap()).is_ok(),
    )))
}

pub fn is_list(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    let vec = expect_args(&obj, "list?", 1)?;
    Ok(Rc::new(Object::Boolean(
        list_to_vec(vec.get(0).unwrap()).is_ok(),
    )))
}

pub fn is_null(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    let vec = expect_args(&obj, "null?", 1)?;
    Ok(Rc::new(Object::Boolean(vec.get(0).unwrap().is_nil())))
}
