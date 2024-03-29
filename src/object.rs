use crate::functions::Function;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Number {
    Float(f64),
    Integer(i64),
}

#[derive(PartialEq)]
pub enum Object {
    Nil,
    Boolean(bool),
    Symbol(String),
    String(String),
    Number(Number),
    Pair(Rc<Object>, Rc<Object>),
    Function(Function),
}

impl Object {
    pub fn make_pair(a: Object, b: Object) -> Object {
        Object::Pair(Rc::new(a), Rc::new(b))
    }
    pub fn make_int(value: i64) -> Object {
        Object::Number(Number::Integer(value))
    }
    pub fn is_nil(&self) -> bool {
        *self == Object::Nil
    }
    pub fn is_true(&self) -> bool {
        *self != Object::Boolean(false)
    }
}

#[rustfmt::skip]
impl Debug for Object {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Object::Nil => write!(f, "()"),
            Object::Boolean(b) => write!(f, "{}", (if *b { "#t" } else { "#f" })),
            Object::Symbol(s) => write!(f, "{}", s),
            Object::String(s) => write!(f, "\"{}\"", s),
            Object::Number(Number::Float(v)) => write!(f, "{}", v),
            Object::Number(Number::Integer(v)) => write!(f, "{}", v),
            Object::Pair(a, b) => write!(f, "({:?} . {:?})", a, b),
            Object::Function(_) => write!(f, "<function>"),
        }
    }
}

/// Display provides prettier output of lists then Debug
#[rustfmt::skip]
impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if let Object::Pair(head, tail) = self {
            let mut s = String::new();
            let mut obj = tail.as_ref();
            while let Object::Pair(car, cdr) = obj {
                s += &format!(" {}", car);
                obj = cdr.as_ref();
            }
            if !obj.is_nil() {
                s += &(format!(" . {:?}", obj));
            }
            write!(f, "({}{})", head, s)
        } else {
            write!(f, "{:?}", self)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_format() {
        assert_eq!(format!("{}", Object::Nil), "()");

        let obj = Object::make_pair(Object::Boolean(true), Object::Boolean(false));
        assert_eq!(format!("{}", obj), "(#t . #f)");

        let obj = Object::make_pair(Object::make_int(1), Object::Nil);
        assert_eq!(format!("{}", obj), "(1)");

        let obj = Object::make_pair(Object::make_int(1), Object::make_int(2));
        assert_eq!(format!("{}", obj), "(1 . 2)");

        let obj = Object::make_pair(
            Object::make_int(1),
            Object::make_pair(Object::make_int(2), Object::Nil),
        );
        assert_eq!(format!("{}", obj), "(1 2)");
    }
}
