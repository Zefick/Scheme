
use crate::scope::Scope;
use crate::object::Object;
use crate::eval::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;


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
                fn_begin(body, &Rc::clone(scope))
            }
        }
    }
    pub fn new(name: &String, args: &Rc<Object>, body: &Rc<Object>, scope: &Rc<RefCell<Scope>>)
            -> Result<Object, String> {
        let mut list = args;
        let mut vec = Vec::new();
        while let Object::Pair(head, tail) = list.as_ref() {
            vec.push(head);
            list = tail;
        }
        if !list.is_nil() {
            vec.push(list);
        }
        let mut ids = HashSet::<&String>::new();
        for id in vec {
            if let Object::Symbol(s) = id.as_ref() {
                if ids.contains(s) {
                    return Err(format!("duplication of argument id '{}'", s).to_string())
                }
                ids.insert(s);
            } else {
                return Err(format!("expected symbol for argument id, found '{}'", id).to_string())
            }
        }
        Ok(Object::Function(Function::Object {
            name: name.clone(),
            args: Rc::clone(args),
            body: Rc::clone(body),
            scope: Rc::clone(scope)
        }))
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
