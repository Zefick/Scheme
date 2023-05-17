use crate::errors::EvalErr;
use crate::object::Number::{Float, Integer};
use crate::object::Object::Boolean;
use crate::object::{Number, Object};
use crate::service::{expect_1_arg, expect_2_args};

use std::rc::Rc;

pub fn is_number(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let arg = expect_1_arg(args, "number?")?;
    Ok(Rc::new(Boolean(matches!(arg.as_ref(), Object::Number(_)))))
}

pub fn is_integer(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let arg = expect_1_arg(args, "integer?")?;
    Ok(Rc::new(Boolean(matches!(
        arg.as_ref(),
        Object::Number(Number::Integer(_))
    ))))
}

pub fn is_real(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    is_number(args)
}

fn get_float(num: &Number) -> f64 {
    match num {
        Integer(x) => *x as f64,
        Float(x) => *x,
    }
}

fn num_predicate(
    vec: Vec<Rc<Object>>, name: &str, f: fn(&Number, &Number) -> bool,
) -> Result<Rc<Object>, EvalErr> {
    if vec.len() < 2 {
        Err(EvalErr::NeedAtLeastArgs(name.to_string(), 2, vec.len()))
    } else {
        if let Object::Number(first) = vec[0].as_ref() {
            let mut last = first;
            let mut result = true;
            for x in &vec[1..] {
                if let Object::Number(x) = x.as_ref() {
                    if !f(last, x) {
                        result = false;
                        break;
                    }
                    last = x;
                } else {
                    return Err(EvalErr::NumericArgsRequiredFor(name.to_string()));
                }
            }
            Ok(Rc::new(Boolean(result)))
        } else {
            Err(EvalErr::NumericArgsRequiredFor(name.to_string()))
        }
    }
}

pub fn num_equal(n1: &Number, n2: &Number) -> bool {
    match (n1, n2) {
        (Integer(a), Integer(b)) => a == b,
        (a, b) => get_float(a) == get_float(b),
    }
}

pub fn num_eqv(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    num_predicate(args, "=", num_equal)
}

pub fn num_less(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    num_predicate(args, "<", |x, y| get_float(x) < get_float(y))
}

pub fn num_greater(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    num_predicate(args, ">", |x, y| get_float(x) > get_float(y))
}

pub fn num_plus(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let mut acc = Integer(0);
    for n in args {
        if let Object::Number(n) = n.as_ref() {
            acc = match (acc, n) {
                (Integer(a), Integer(b)) => Integer(a + *b),
                (a, b) => Float(get_float(&a) + get_float(b)),
            }
        } else {
            return Err(EvalErr::NumericArgsRequiredFor("+".to_string()));
        }
    }
    Ok(Rc::new(Object::Number(acc)))
}

pub fn num_mul(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let mut acc = Integer(1);
    for n in args {
        if let Object::Number(n) = n.as_ref() {
            acc = match (acc, n) {
                (Integer(a), Integer(b)) => Integer(a * *b),
                (a, b) => Float(get_float(&a) * get_float(b)),
            }
        } else {
            return Err(EvalErr::NumericArgsRequiredFor("*".to_string()));
        }
    }
    Ok(Rc::new(Object::Number(acc)))
}

pub fn num_minus(vec: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let mut result = Integer(0);
    for n in 0..vec.len() {
        if let Object::Number(x) = vec.get(n).unwrap().as_ref() {
            if n == 0 && vec.len() > 1 {
                result = *x;
            } else {
                result = match (result, x) {
                    (Integer(a), Integer(b)) => Integer(a - *b),
                    (a, b) => Float(get_float(&a) - get_float(b)),
                };
            }
        } else {
            return Err(EvalErr::NumericArgsRequiredFor("-".to_string()));
        }
    }
    Ok(Rc::new(Object::Number(result)))
}

pub fn num_div(vec: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let mut result = 1.0;
    for n in 0..vec.len() {
        if let Object::Number(x) = vec[n].as_ref() {
            if n == 0 && vec.len() > 1 {
                result = get_float(x);
            } else {
                let x = get_float(x);
                if x == 0.0 {
                    return Err(EvalErr::DivisionByZero());
                }
                result /= x;
            }
        } else {
            return Err(EvalErr::NumericArgsRequiredFor("/".to_string()));
        }
    }
    Ok(Rc::new(Object::Number(Float(result))))
}

fn check_int_div(vec: Vec<Rc<Object>>, name: &str) -> Result<(i64, i64), EvalErr> {
    let (n, d) = expect_2_args(vec, name)?;
    if let (Object::Number(Integer(n)), Object::Number(Integer(d))) = (n.as_ref(), d.as_ref()) {
        if *d == 0 {
            return Err(EvalErr::DivisionByZero());
        }
        Ok((*n, *d))
    } else {
        Err(EvalErr::IntegerArgsRequiredFor(name.to_string()))
    }
}

pub fn quotient(vec: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let (n, d) = check_int_div(vec, "quotient")?;
    Ok(Rc::new(Object::Number(Integer(n / d))))
}

pub fn remainder(vec: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let (n, d) = check_int_div(vec, "remainder")?;
    Ok(Rc::new(Object::Number(Integer(n - n / d * d))))
}

pub fn modulo(vec: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    let (n, d) = check_int_div(vec, "modulo")?;
    let q = ((n as f64) / (d as f64)).floor() as i64;
    let rem = n - d * q;
    Ok(Rc::new(Object::Number(Integer(rem))))
}
