use crate::eval::{expect_args, list_to_vec};
use crate::object::Number::{Float, Integer};
use crate::object::Object::Boolean;
use crate::object::{Number, Object};

use std::rc::Rc;

fn normalize(x: Number) -> Number {
    if let Number::Float(f) = x {
        if f.floor() == f {
            return Number::Integer(f as i32);
        }
    }
    x.clone()
}

pub fn is_number(args: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(&args, "number?", 1).and_then(|vec| {
        return Ok(Rc::new(Object::Boolean(
            match vec.get(0).unwrap().as_ref() {
                Object::Number(_) => true,
                _ => false,
            },
        )));
    })
}

pub fn is_integer(args: Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(&args, "integer?", 1).and_then(|vec| {
        return Ok(Rc::new(Object::Boolean(
            match vec.get(0).unwrap().as_ref() {
                Object::Number(Number::Integer(_)) => true,
                _ => false,
            },
        )));
    })
}

pub fn is_real(args: Rc<Object>) -> Result<Rc<Object>, String> {
    is_number(args)
}

fn get_float(num: &Number) -> f64 {
    return match num {
        Integer(x) => *x as f64,
        Float(x) => *x,
    };
}

fn check_nums<'a, 'b>(x: &'a Object, y: &'b Object) -> Result<(&'a Number, &'b Number), String> {
    match (x, y) {
        (Object::Number(a), Object::Number(b)) => Ok((a, b)),
        _ => Err("number arguments required".to_string()),
    }
}

fn num_predicate(
    args: Rc<Object>,
    name: &'static str,
    f: fn(&Number, &Number) -> bool,
) -> Result<Rc<Object>, String> {
    list_to_vec(&args).and_then(|vec| {
        if vec.len() < 2 {
            return Err("'=' need at least 2 arguments".to_string());
        } else {
            let mut result = true;
            for i in 0..vec.len() - 1 {
                match check_nums(vec.get(i).unwrap(), vec.get(i + 1).unwrap()) {
                    Ok((x, y)) => result = f(x, y),
                    Err(_) => {
                        return Err(format!("numeric arguments required for '{}'", name).to_string())
                    }
                }
                if !result {
                    break;
                }
            }
            return Ok(Rc::new(Boolean(result)));
        }
    })
}

pub fn num_eqv(args: Rc<Object>) -> Result<Rc<Object>, String> {
    num_predicate(args, "=", |x, y| match (x, y) {
        (Integer(a), Integer(b)) => a == b,
        (a, b) => get_float(a) == get_float(b),
    })
}

pub fn num_less(args: Rc<Object>) -> Result<Rc<Object>, String> {
    num_predicate(args, "<", |x, y| match (x, y) {
        (Integer(a), Integer(b)) => a < b,
        (a, b) => get_float(a) < get_float(b),
    })
}

pub fn num_greater(args: Rc<Object>) -> Result<Rc<Object>, String> {
    num_predicate(args, ">", |x, y| match (x, y) {
        (Integer(a), Integer(b)) => a > b,
        (a, b) => get_float(a) > get_float(b),
    })
}

pub fn num_plus(args: Rc<Object>) -> Result<Rc<Object>, String> {
    list_to_vec(&args).and_then(|vec| {
        let mut acc = Number::Integer(0);
        for n in vec {
            if let Object::Number(n) = n.as_ref() {
                acc = match (acc, n) {
                    (Number::Integer(a), Number::Integer(b)) => Number::Integer(a + *b),
                    (a, b) => Number::Float(get_float(&a) + get_float(b)),
                }
            } else {
                return Err("numeric arguments required for '+'".to_string());
            }
        }
        Ok(Rc::new(Object::Number(acc)))
    })
}

pub fn num_mul(args: Rc<Object>) -> Result<Rc<Object>, String> {
    list_to_vec(&args).and_then(|vec| {
        let mut acc = Number::Integer(1);
        for n in vec {
            if let Object::Number(n) = n.as_ref() {
                acc = match (acc, n) {
                    (Number::Integer(a), Number::Integer(b)) => Number::Integer(a * *b),
                    (a, b) => Number::Float(get_float(&a) * get_float(b)),
                }
            } else {
                return Err("numeric arguments required for '*'".to_string());
            }
        }
        Ok(Rc::new(Object::Number(acc)))
    })
}

pub fn num_minus(args: Rc<Object>) -> Result<Rc<Object>, String> {
    list_to_vec(&args).and_then(|vec| {
        let mut result = Number::Integer(0);
        for n in 0..vec.len() {
            if let Object::Number(x) = vec.get(n).unwrap().as_ref() {
                if n == 0 && vec.len() > 1 {
                    result = x.clone();
                } else {
                    result = match (result, x) {
                        (Number::Integer(a), Number::Integer(b)) => Number::Integer(a - *b),
                        (a, b) => Number::Float(get_float(&a) - get_float(b)),
                    };
                }
            } else {
                return Err("numeric arguments required for '-'".to_string());
            }
        }
        Ok(Rc::new(Object::Number(result)))
    })
}

pub fn num_div(args: Rc<Object>) -> Result<Rc<Object>, String> {
    list_to_vec(&args).and_then(|vec| {
        let mut result = Number::Integer(1);
        for n in 0..vec.len() {
            if let Object::Number(x) = vec.get(n).unwrap().as_ref() {
                if n == 0 && vec.len() > 1 {
                    result = x.clone();
                } else {
                    if match x {
                        Integer(0) => true,
                        Float(x) => *x == 0.0,
                        _ => false,
                    } {
                        return Err("division by 0".to_string());
                    }
                    result = Number::Float(get_float(&result) / get_float(x));
                }
            } else {
                return Err("numeric arguments required for '/'".to_string());
            }
        }
        Ok(Rc::new(Object::Number(normalize(result))))
    })
}
