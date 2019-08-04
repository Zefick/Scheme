
use std::rc::Rc;
use std::fmt::{Debug, Display, Formatter};


#[derive(PartialEq, Debug)]
pub enum Number {
    Float(f64),
    Integer(i32)
}

#[derive(PartialEq)]
pub enum Object {
    Nil,
    Boolean(bool),
    Symbol(String),
    String(String),
    Number(Number),
    Pair(Rc<Object>, Rc<Object>),
    Function(fn(Rc<Object>) -> Result<Rc<Object>, String>),
}

impl Object {
    pub fn make_pair(a : Object, b : Object) -> Object {
        Object::Pair(Rc::new(a), Rc::new(b))
    }
    pub fn make_int(value : i32) -> Object {
        Object::Number(Number::Integer(value))
    }
    pub fn is_nil(&self) -> bool {
        *self == Object::Nil
    }
    pub fn is_true(&self) -> bool {
        *self != Object::Boolean(false)
    }
}

impl Default for Object {
    fn default() -> Self {Object::Nil}
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Object::Nil => write!(f, "()"),
            Object::Boolean(b) => write!(f, "{}", (if *b {"#t"} else {"#f"})),
            Object::Symbol(s) => write!(f, "{}", s),
            Object::String(s) => write!(f, "\"{}\"", s),
            Object::Number(Number::Float(v)) => write!(f, "{}", v),
            Object::Number(Number::Integer(v)) => write!(f, "{}", v),
            Object::Pair(a, b) => write!(f, "({:?} . {:?})", a, b),
            Object::Function(_) => write!(f, "<function>")
        }
    }
}

/// Display provides prettier output of lists then Debug
impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Object::Pair(head, tail) => {
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
            },
            _ => (self as &Debug).fmt(f)
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

        let obj = Object::make_pair(Object::make_int(1),
                                    Object::make_pair(Object::make_int(2), Object::Nil));
        assert_eq!(format!("{}", obj), "(1 2)");
    }

}