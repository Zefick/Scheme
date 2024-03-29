use crate::errors::EvalErr;
use crate::object::Object;
use crate::object::Object::Pair;
use crate::scope::Scope;
use crate::service::{list_to_vec, vec_to_list};
use crate::eval::{eval, fn_begin};

use std::collections::HashSet;
use std::rc::Rc;

type RustFn = fn(Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr>;

pub enum Function {
    Dynamic(String),
    Pointer(RustFn),
    Object {
        name: String,
        args: Rc<Object>,
        body: Vec<Rc<Object>>,
        scope: Rc<Scope>,
    },
}

pub enum CallResult {
    Object(Rc<Object>),
    TailCall(Rc<Object>, Rc<Scope>),
}

impl Function {
    pub fn call(&self, args: Vec<Rc<Object>>) -> Result<CallResult, EvalErr> {
        match self {
            Function::Dynamic(s) => Ok(CallResult::Object(crate::lists::cadr(s, args)?)),

            Function::Pointer(f) => Ok(CallResult::Object(f(args)?)),

            Function::Object { name, args: formal_args, body, scope } => {
                let mut formals = formal_args;
                let scope = Scope::from_scope(scope);
                let mut arg_num = 0;
                loop {
                    match formals.as_ref() {
                        Pair(a1, b1) => {
                            if args.len() <= arg_num {
                                return Err(EvalErr::TooFewArguments(name.to_string()));
                            }
                            if let Object::Symbol(s) = a1.as_ref() {
                                scope.bind(s, args[arg_num].clone());
                                formals = b1;
                            } else {
                                panic!("unexpected branch");
                            }
                        }
                        Object::Symbol(s) => {
                            scope.bind(s, Rc::new(vec_to_list(&args[arg_num..])));
                            break;
                        }
                        Object::Nil => {
                            if args.len() > arg_num {
                                return Err(EvalErr::TooManyArguments(name.to_string()));
                            }
                            break;
                        }
                        _ => return Err(EvalErr::WrongArgsList(vec_to_list(&args).to_string())),
                    }
                    arg_num += 1;
                }
                fn_begin(body, &Rc::new(scope))
            }
        }
    }

    fn check_args(args: &Rc<Object>) -> Result<(), EvalErr> {
        let mut list = args;
        let mut vec = Vec::new();
        while let Pair(head, tail) = list.as_ref() {
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
        name: String, args: Rc<Object>, body: Vec<Rc<Object>>, scope: Rc<Scope>,
    ) -> Result<Object, EvalErr> {
        Function::check_args(&args)?;
        if body.is_empty() {
            return Err(EvalErr::EmptyFunctionBody());
        }
        let func = Function::Object { name, args, body, scope };
        Ok(Object::Function(func))
    }

    pub fn from_pointer(f: RustFn) -> Object {
        Object::Function(Function::Pointer(f))
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Dynamic(s1), Function::Dynamic(s2)) => s1 == s2,
            (Function::Pointer(f1), Function::Pointer(f2)) => f1 == f2,
            _ => std::ptr::eq(self, other),
        }
    }
}

pub fn fn_apply(vec: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<CallResult, EvalErr> {
    if vec.len() < 2 {
        return Err(EvalErr::NeedAtLeastArgs("apply".to_string(), 2, vec.len()));
    }
    let first = eval(&vec[0], scope)?;
    if let Object::Function(fun) = first.as_ref() {
        // concatenate first arguments with the last one presented as a list
        // e.g. (1 2 3 '(4 5)) => (1 2 3 4 5)
        let args = eval(vec.last().unwrap(), scope)?;
        if let Ok(last) = list_to_vec(args.as_ref()) {
            let mut args: Vec<Rc<Object>> = vec![];
            for arg in vec[1..vec.len() - 1].iter() {
                args.push(eval(arg, scope)?);
            }
            args.extend(last);
            Ok(fun.call(args)?)
        } else {
            Err(EvalErr::ApplyNeedsProperList(args.to_string()))
        }
    } else {
        Err(EvalErr::IllegalObjectAsAFunction(first.to_string()))
    }
}

pub fn fn_map(vec: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    if vec.len() < 2 {
        return Err(EvalErr::NeedAtLeastArgs("map".to_string(), 2, vec.len()));
    }
    let func = &vec[0];
    if let Object::Function(f) = func.as_ref() {
        // first check that all arguments are lists of the same size
        let mut len = None;
        let mut inputs = Vec::new();
        for arg in vec[1..].iter() {
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
            let args = inputs.iter().map(|v| v[i].clone()).collect();
            match f.call(args)? {
                CallResult::Object(obj) => result.push(obj),
                CallResult::TailCall(obj, scope) => result.push(eval(&obj, &scope)?),
            };
        }
        Ok(Rc::new(vec_to_list(&result)))
    } else {
        Err(EvalErr::IllegalObjectAsAFunction(func.to_string()))
    }
}
