use crate::errors::EvalErr;
use crate::object::Number::{Float, Integer};
use crate::object::Object::Boolean;
use crate::object::{Number, Object};
use crate::service::*;

use std::rc::Rc;

fn normalize(x: Number) -> Number {
    if let Number::Float(f) = x {
        if f.floor() == f {
            return Number::Integer(f as i32);
        }
    }
    x.clone()
}

pub fn is_number(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    let arg = expect_1_arg(&args, "number?")?;
    Ok(Rc::new(Object::Boolean(match arg.as_ref() {
        Object::Number(_) => true,
        _ => false,
    })))
}

pub fn is_integer(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    let arg = expect_1_arg(&args, "integer?")?;
    Ok(Rc::new(Object::Boolean(match arg.as_ref() {
        Object::Number(Number::Integer(_)) => true,
        _ => false,
    })))
}

pub fn is_real(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    is_number(args)
}

fn get_float(num: &Number) -> f64 {
    return match num {
        Integer(x) => *x as f64,
        Float(x) => *x,
    };
}

fn check_nums<'a, 'b>(x: &'a Object, y: &'b Object) -> Result<(&'a Number, &'b Number), ()> {
    match (x, y) {
        (Object::Number(a), Object::Number(b)) => Ok((a, b)),
        _ => Err(()),
    }
}

fn num_predicate(
    args: Rc<Object>,
    name: &'static str,
    f: fn(&Number, &Number) -> bool,
) -> Result<Rc<Object>, EvalErr> {
    let vec = list_to_vec(&args)?;
    if vec.len() < 2 {
        Err(EvalErr::NeedAtLeastArgs(name.to_string(), 2, vec.len()))
    } else {
        let mut result = true;
        for i in 0..vec.len() - 1 {
            if !check_nums(vec.get(i).unwrap(), vec.get(i + 1).unwrap())
                .map_err(|_| EvalErr::NumericArgsRequiredFor(name.to_string()))
                .map(|(x, y)| f(x, y))?
            {
                result = false;
                break;
            }
        }
        Ok(Rc::new(Boolean(result)))
    }
}

pub fn num_equal(n1: &Number, n2: &Number) -> bool {
    match (n1, n2) {
        (Integer(a), Integer(b)) => a == b,
        (a, b) => get_float(a) == get_float(b),
    }
}

pub fn num_eqv(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    num_predicate(args, "=", num_equal)
}

pub fn num_less(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    num_predicate(args, "<", |x, y| match (x, y) {
        (Integer(a), Integer(b)) => a < b,
        (a, b) => get_float(a) < get_float(b),
    })
}

pub fn num_greater(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    num_predicate(args, ">", |x, y| match (x, y) {
        (Integer(a), Integer(b)) => a > b,
        (a, b) => get_float(a) > get_float(b),
    })
}

pub fn num_plus(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    let mut acc = Number::Integer(0);
    for n in list_to_vec(&args)? {
        if let Object::Number(n) = n.as_ref() {
            acc = match (acc, n) {
                (Number::Integer(a), Number::Integer(b)) => Number::Integer(a + *b),
                (a, b) => Number::Float(get_float(&a) + get_float(b)),
            }
        } else {
            return Err(EvalErr::NumericArgsRequiredFor("+".to_string()));
        }
    }
    Ok(Rc::new(Object::Number(acc)))
}

pub fn num_mul(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    let mut acc = Number::Integer(1);
    for n in list_to_vec(&args)? {
        if let Object::Number(n) = n.as_ref() {
            acc = match (acc, n) {
                (Number::Integer(a), Number::Integer(b)) => Number::Integer(a * *b),
                (a, b) => Number::Float(get_float(&a) * get_float(b)),
            }
        } else {
            return Err(EvalErr::NumericArgsRequiredFor("*".to_string()));
        }
    }
    Ok(Rc::new(Object::Number(acc)))
}

pub fn num_minus(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    let vec = list_to_vec(&args)?;
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
            return Err(EvalErr::NumericArgsRequiredFor("-".to_string()));
        }
    }
    Ok(Rc::new(Object::Number(result)))
}

pub fn num_div(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    let vec = list_to_vec(&args)?;
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
                    return Err(EvalErr::DivisionByZero());
                }
                result = Number::Float(get_float(&result) / get_float(x));
            }
        } else {
            return Err(EvalErr::NumericArgsRequiredFor("/".to_string()));
        }
    }
    Ok(Rc::new(Object::Number(normalize(result))))
}
