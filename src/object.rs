
use std::boxed::Box;
use std::fmt::Debug;
use std::fmt::Formatter;


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
    Pair(Box<Object>, Box<Object>)
}

impl Object {
    pub fn make_pair(a : Object, b : Object) -> Object {
        return Object::Pair(Box::new(a), Box::new(b));
    }
}

impl Default for Object {
    fn default() -> Self {Object::Nil}
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Object::Nil => write!(f, "Nil"),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Symbol(s) => write!(f, "{}", s),
            Object::String(s) => write!(f, "\"{}\"", s),
            Object::Number(Number::Float(v)) => write!(f, "{}", v),
            Object::Number(Number::Integer(v)) => write!(f, "{}", v),
            Object::Pair(a, b) => write!(f, "({:?} . {:?})", a, b)
        }
    }
}