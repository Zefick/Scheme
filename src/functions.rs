use crate::eval::*;
use crate::object::Object;
use crate::scope::Scope;
use crate::service::{list_to_vec, vec_to_list};

use crate::errors::EvalErr;
use std::collections::HashSet;
use std::rc::Rc;

pub enum Function {
    Dynamic(String),
    Pointer(fn(Rc<Object>) -> Result<Rc<Object>, EvalErr>),
    Object {
        name: String,
        args: Rc<Object>,
        body: Rc<Object>,
        scope: Rc<Scope>,
    },
}

impl Function {
    pub fn call(&self, args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
        match self {
            Function::Dynamic(s) => crate::lists::cadr(s, &args),
            Function::Pointer(f) => f(args),
            Function::Object {
                name,
                args: formal_args,
                body,
                scope,
            } => {
                let mut formals = formal_args;
                let mut rest = &args;
                let scope = &Scope::new(&[], Some(&Rc::clone(scope)));
                loop {
                    match (formals.as_ref(), rest.as_ref()) {
                        (Object::Pair(a1, b1), Object::Pair(a2, b2)) => {
                            if let Object::Symbol(s) = a1.as_ref() {
                                scope.bind(s, Rc::clone(a2));
                                formals = b1;
                                rest = b2;
                            } else {
                                panic!("unexpected branch");
                            }
                        }
                        (Object::Symbol(s), _) => {
                            scope.bind(s, Rc::clone(rest));
                            break;
                        }
                        (Object::Nil, Object::Nil) => {
                            break;
                        }
                        (_, Object::Nil) => return Err(EvalErr::TooFewArguments(name.to_string())),
                        (Object::Nil, _) => {
                            return Err(EvalErr::TooManyArguments(name.to_string()))
                        }
                        _ => return Err(EvalErr::WrongArgsList(args.to_string())),
                    }
                }
                fn_begin(body, &Rc::clone(scope))
            }
        }
    }

    fn check_args(args: &Rc<Object>) -> Result<(), EvalErr> {
        let mut list = args;
        let mut vec = Vec::new();
        while let Object::Pair(head, tail) = list.as_ref() {
            vec.push(head);
            list = tail;
        }
        if !list.is_nil() {
            vec.push(list);
        }
        let mut ids = HashSet::new();
        for id in vec {
            if let Object::Symbol(s) = id.as_ref() {
                if ids.contains(s) {
                    return Err(EvalErr::ArgumentDuplication(s.to_string()));
                }
                ids.insert(s);
            } else {
                return Err(EvalErr::ExpectedSymbolForArgument(id.to_string()));
            }
        }
        Ok(())
    }

    pub fn new(
        name: &String,
        args: &Rc<Object>,
        body: &Rc<Object>,
        scope: &Rc<Scope>,
    ) -> Result<Object, EvalErr> {
        Function::check_args(args)?;
        Ok(Object::Function(Function::Object {
            name: name.clone(),
            args: Rc::clone(args),
            body: Rc::clone(body),
            scope: Rc::clone(scope),
        }))
    }
}

impl PartialEq<Self> for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Dynamic(s1), Function::Dynamic(s2)) => s1 == s2,
            (Function::Pointer(f1), Function::Pointer(f2)) => f1 == f2,
            _ => std::ptr::eq(self, other),
        }
    }
}

pub fn fn_apply(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    let vec = list_to_vec(args.as_ref())?;
    if vec.len() < 2 {
        return Err(EvalErr::NeedAtLeastArgs("apply".to_string(), 2, vec.len()));
    }
    let func = vec.get(0).unwrap();
    if let Object::Function(f) = func.as_ref() {
        let mut args = Rc::clone(vec.get(vec.len() - 1).unwrap());
        if list_to_vec(&args).is_err() {
            return Err(EvalErr::ApplyNeedsProperList(args.to_string()));
        }
        // concatenate first arguments with a last one presented as a list
        for i in (1..vec.len() - 1).rev() {
            args = Rc::new(Object::Pair(Rc::clone(vec.get(i).unwrap()), args));
        }
        f.call(args)
    } else {
        Err(EvalErr::IllegalObjectAsAFunction(func.to_string()))
    }
}

pub fn fn_map(args: Rc<Object>) -> Result<Rc<Object>, EvalErr> {
    let vec = list_to_vec(args.as_ref())?;
    if vec.len() < 2 {
        return Err(EvalErr::NeedAtLeastArgs("map".to_string(), 2, vec.len()));
    }
    let func = vec.get(0).unwrap();
    if let Object::Function(f) = func.as_ref() {
        // first check that all arguments are lists and have the same size
        let mut len = None;
        let mut inputs = Vec::new();
        for arg in vec.iter().skip(1) {
            let vec = list_to_vec(arg)?;
            if len.is_none() {
                len = Some(vec.len());
            } else if len.unwrap() != vec.len() {
                return Err(EvalErr::UnequalMapLists());
            }
            inputs.push(vec);
        }
        // then call a mapped function
        let mut result = Vec::new();
        for i in 0..len.unwrap() {
            let args = inputs.iter().map(|v| v.get(i).unwrap()).cloned().collect();
            result.push(f.call(Rc::new(vec_to_list(args)))?);
        }
        Ok(Rc::new(vec_to_list(result)))
    } else {
        Err(EvalErr::IllegalObjectAsAFunction(func.to_string()))
    }
}
