use crate::object::Object;
use crate::service::*;

use std::rc::Rc;

/// The family of functions which names match with pattern `c[ad]{1,4}r`.
pub fn cadr(name: &str, obj: &Rc<Object>) -> Result<Rc<Object>, String> {
    let mut obj = expect_1_arg(&obj, name)?;
    for op in name[1..name.len() - 1].chars().rev() {
        let pair = check_pair(&obj)?;
        obj = Rc::clone(if op == 'a' { pair.0 } else { pair.1 });
    }
    return Ok(obj);
}

pub fn cons(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    let (car, cdr) = expect_2_args(&obj, "cons")?;
    Ok(Rc::new(Object::Pair(car, cdr)))
}

pub fn list(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    Ok(obj)
}

pub fn length(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    let len = list_to_vec(expect_1_arg(&obj, "length")?.as_ref())?.len();
    Ok(Rc::new(Object::make_int(len as i32)))
}

pub fn is_pair(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    let arg = expect_1_arg(&obj, "pair?")?;
    Ok(Rc::new(Object::Boolean(check_pair(&arg).is_ok())))
}

pub fn is_list(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    let arg = expect_1_arg(&obj, "list?")?;
    Ok(Rc::new(Object::Boolean(list_to_vec(&arg).is_ok())))
}

pub fn is_null(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    let arg = expect_1_arg(&obj, "null?")?;
    Ok(Rc::new(Object::Boolean(arg.is_nil())))
}
