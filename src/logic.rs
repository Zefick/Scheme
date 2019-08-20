use crate::eval::*;
use crate::object::Object;
use crate::scope::Scope;

use std::rc::Rc;

pub fn fn_if(args: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    expect_args(args, "if", 3).and_then(|vec| {
        let mut vec = vec.into_iter();
        eval(&vec.next().unwrap(), scope).and_then(|val| {
            if !val.is_true() {
                vec.next();
            }
            eval(&vec.next().unwrap(), scope)
        })
    })
}

pub fn cond(args: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    list_to_vec(args).and_then(|cond_list| {
        if cond_list.is_empty() {
            return Err(format!("'cond' need at least 1 clause"));
        }
        for condition in cond_list {
            let vec = list_to_vec(condition.as_ref());
            if let Ok(vec) = vec {
                if vec.is_empty() {
                    return Err(format!("empty clause for 'cond'"));
                }
                let predicate = vec.get(0).unwrap();
                let mut is_true = false;
                if let Object::Symbol(s) = predicate.as_ref() {
                    is_true = s == "else";
                } else {
                    let predicate = eval(predicate, scope);
                    if let Ok(x) = predicate {
                        if x.is_true() {
                            if vec.len() == 1 {
                                return Ok(Rc::clone(&x));
                            }
                            is_true = true;
                        }
                    } else {
                        return predicate;
                    }
                }
                if is_true {
                    return fn_begin_vec(vec.into_iter().skip(1), scope);
                }
            } else {
                return Err(vec.unwrap_err());
            }
        }
        Ok(undef())
    })
}

pub fn is_boolean(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(&obj, "boolean?", 1).and_then(|vec| {
        Ok(Rc::new(Object::Boolean(
            match vec.get(0).unwrap().as_ref() {
                Object::Boolean(_) => true,
                _ => false,
            },
        )))
    })
}

pub fn logic_not(obj: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(&obj, "not", 1)
        .and_then(|vec| Ok(Rc::new(Object::Boolean(!vec.get(0).unwrap().is_true()))))
}

pub fn logic_and(obj: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    list_to_vec(&obj).and_then(|vec| {
        let mut result = Rc::new(Object::Boolean(true));
        for obj in vec {
            match eval(&obj, scope) {
                Err(e) => return Err(e),
                Ok(x) => {
                    if x.is_true() {
                        result = x;
                    } else {
                        return Ok(x);
                    }
                }
            }
        }
        return Ok(result);
    })
}

pub fn logic_or(obj: &Object, scope: &Rc<Scope>) -> Result<Rc<Object>, String> {
    list_to_vec(obj).and_then(|vec| {
        for obj in vec {
            match eval(&obj, scope) {
                Err(e) => return Err(e),
                Ok(x) => {
                    if x.is_true() {
                        return Ok(x);
                    }
                }
            }
        }
        return Ok(Rc::new(Object::Boolean(false)));
    })
}
