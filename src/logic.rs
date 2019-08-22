use crate::eval::*;
use crate::object::Object;
use crate::scope::Scope;

use std::rc::Rc;

pub fn fn_if(args: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    let vec = expect_args(args, "if", 3)?;
    let mut vec = vec.into_iter();
    if !eval(&vec.next().unwrap(), scope)?.is_true() {
        vec.next();
    }
    eval(&vec.next().unwrap(), scope)
}

pub fn cond(args: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    let cond_list = list_to_vec(args)?;
    if cond_list.is_empty() {
        return Err(format!("'cond' need at least 1 clause"));
    }
    for condition in cond_list {
        let vec = list_to_vec(condition.as_ref())?;
        if vec.is_empty() {
            return Err(format!("empty clause for 'cond'"));
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
            return fn_begin_vec(vec.into_iter().skip(1), scope);
        }
    }
    Ok(undef())
}

pub fn is_boolean(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(&obj, "boolean?", 1).map(|vec| {
        Rc::new(Object::Boolean(match vec.get(0).unwrap().as_ref() {
            Object::Boolean(_) => true,
            _ => false,
        }))
    })
}

pub fn logic_not(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(&obj, "not", 1).map(|vec| Rc::new(Object::Boolean(!vec.get(0).unwrap().is_true())))
}

pub fn logic_and(obj: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    let vec = list_to_vec(&obj)?;
    let mut result = Rc::new(Object::Boolean(true));
    for obj in vec {
        let x = eval(&obj, scope)?;
        if x.is_true() {
            result = x;
        } else {
            return Ok(x);
        }
    }
    return Ok(result);
}

pub fn logic_or(obj: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    let vec = list_to_vec(obj)?;
    for obj in vec {
        let x = eval(&obj, scope)?;
        if x.is_true() {
            return Ok(x);
        }
    }
    return Ok(Rc::new(Object::Boolean(false)));
}
