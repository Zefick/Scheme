
use crate::scope::Scope;
use crate::object::Object;
use crate::eval::*;
use std::rc::Rc;
use std::cell::RefCell;


pub enum Function {
    Pointer(fn(Rc<Object>) -> Result<Rc<Object>, String>),
    Object {
        name: String,
        args: Rc<Object>,
        body: Rc<Object>,
        scope: Rc<RefCell<Scope>>
    }
}

impl Function {
    pub fn call(&self, args : Rc<Object>) -> Result<Rc<Object>, String> {
        match self {
            Function::Pointer(f) => {
                f(args)
            },
            Function::Object{name, args: formal_args, body, scope} => {
                let mut formals = formal_args;
                let mut rest = &args;
                loop {
                    match (formals.as_ref(), rest.as_ref()) {
                        (Object::Pair(a1, b1), Object::Pair(a2, b2)) => {
                            if let Object::Symbol(s) = a1.as_ref() {
                                scope.borrow_mut().bind(s.clone(), Rc::clone(a2));
                                formals = b1;
                                rest = b2;
                            } else {
                                panic!("unexpected branch");
                            }
                        },
                        (Object::Symbol(s), _) => {
                            scope.borrow_mut().bind(s.clone(), Rc::clone(rest));
                            break;
                        },
                        (Object::Nil, Object::Nil) => {
                            break;
                        }
                        (_, Object::Nil) => {
                            return Err(format!("too few arguments given to '{}'", name).to_string())
                        },
                        (Object::Nil, _) => {
                            return Err(format!("too many arguments given to '{}'", name).to_string())
                        },
                        _ => return Err(format!("wrong arguments list '{}'", args.as_ref()).to_string())
                    }
                }
                fn_begin(body, Rc::clone(scope))
            }
        }
    }
}

impl PartialEq<Self> for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Pointer(f1), Function::Pointer(f2)) => f1 == f2,
            _ => &self == &other
        }
    }
}

/// Returns a first element of a list or a pair.
pub fn car(obj : Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "car", 1).and_then(|vec| {
        check_pair(vec.get(0).unwrap()).map(|x| Rc::clone(x.0))
    })
}

/// Returns a second element of a pair which is all elements after first for lists.
pub fn cdr(obj : Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "cdr", 1).and_then(|vec| {
        check_pair(vec.get(0).unwrap()).map(|x| Rc::clone(x.1))
    })
}

pub fn cons(obj : Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "cons", 2).map(|vec| {
        Rc::new(Object::Pair(
            Rc::clone(vec.get(0).unwrap()),
            Rc::clone(vec.get(1).unwrap())))
    })
}

pub fn list(obj : Rc<Object>) -> Result<Rc<Object>, String> {
    Ok(obj)
}

pub fn length(obj : Rc<Object>) -> Result<Rc<Object>, String> {
    expect_args(obj.as_ref(), "length", 1).and_then(|vec| {
        list_to_vec(vec.get(0).unwrap())
            .map(|x| Rc::new(Object::make_int(x.len() as i32)))
    })
}
