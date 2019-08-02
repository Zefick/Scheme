
use std::rc::Rc;
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
    Pair(Rc<Object>, Rc<Object>),
    Function(fn(Rc<Object>) -> Result<Rc<Object>, String>),
}

impl Object {
    pub fn make_pair(a : Object, b : Object) -> Object {
        return Object::Pair(Rc::new(a), Rc::new(b));
    }
    pub fn make_int(value : i32) -> Object {
        return Object::Number(Number::Integer(value));
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
            Object::Pair(a, b) => write!(f, "({:?} . {:?})", a, b),
            Object::Function(_) => write!(f, "<function>")
        }
    }
}