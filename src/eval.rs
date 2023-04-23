use crate::errors::EvalErr;
use crate::functions::*;
use crate::logic::*;
use crate::object::*;
use crate::scope::*;
use crate::service::*;

use std::rc::Rc;

pub fn eval_args(args: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<Vec<Rc<Object>>, EvalErr> {
    let mut result = Vec::new();
    for arg in args {
        result.push(eval(&arg, scope)?);
    }
    Ok(result)
}

fn fn_let(
    let_args: Vec<Rc<Object>>, scope: &Rc<Scope>, star: bool, rec: bool,
) -> Result<CallResult, EvalErr> {
    if let_args.len() < 2 {
        return Err(EvalErr::NeedAtLeastArgs(
            "let".to_string(),
            2,
            let_args.len(),
        ));
    }
    let root_scope = Rc::new(Scope::from_scope(scope));
    let mut init_scope = root_scope.clone();
    let args = list_to_vec(let_args.get(0).unwrap())?;
    let mut bindings = vec![];
    for arg in args {
        let init_expr = list_to_vec(arg.as_ref())?;
        if init_expr.len() >= 2 {
            let var = init_expr.get(0).unwrap().as_ref();
            if let Object::Symbol(s) = var {
                let value = eval(init_expr.get(1).unwrap(), &init_scope)?;
                if star {
                    init_scope = Rc::new(Scope::from_scope(&init_scope));
                    init_scope.bind(s, value.clone());
                }
                if rec {
                    root_scope.bind(s, value.clone());
                }
                bindings.push((s.to_string(), value.clone()));
            } else {
                return Err(EvalErr::LetNeedSymbolForBinding(var.to_string()));
            }
        } else {
            return Err(EvalErr::LetNeedListForBinding(arg.to_string()));
        }
    }
    fn_begin(
        &let_args[1..],
        &Rc::new(Scope::new(bindings.as_slice(), scope)),
    )
}

pub fn fn_begin(args: &[Rc<Object>], scope: &Rc<Scope>) -> Result<CallResult, EvalErr> {
    if args.is_empty() {
        return Ok(CallResult::Object(undef()));
    }
    for arg in &args[..args.len() - 1] {
        eval(arg, scope)?;
    }
    Ok(CallResult::TailCall(
        args[args.len() - 1].clone(),
        scope.clone(),
    ))
}

/// There are two forms of define:
///
/// `(define id expr)` and `(define (id args...) expr...)`
///
/// Both syntax handled by this function. Defined value added to a current scope
fn fn_define(args: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<(), EvalErr> {
    let head = args[0].clone();
    match head.as_ref() {
        // (define x expr)
        Object::Symbol(s) => {
            if args.len() > 2 {
                Err(EvalErr::TooManyArguments("define".to_string()))
            } else {
                scope.bind(s, eval(&args[1], scope)?);
                Ok(())
            }
        }
        // (define (name args) body)
        Object::Pair(name, fun_args) => {
            if let Object::Symbol(s) = name.as_ref() {
                scope.bind(
                    s,
                    Rc::new(Function::new(
                        s.clone(),
                        Rc::clone(fun_args),
                        args[1..].to_vec(),
                        Rc::clone(scope),
                    )?),
                );
                Ok(())
            } else {
                Err(EvalErr::ExpectedSymbolForFunctionName(name.to_string()))
            }
        }
        x => Err(EvalErr::WrongDefineArgument(x.to_string())),
    }
}

fn lambda(args: Vec<Rc<Object>>, scope: &Rc<Scope>) -> Result<Rc<Object>, EvalErr> {
    Ok(Rc::new(Function::new(
        "#<lambda>".to_string(),
        Rc::clone(&args[0]),
        args[1..].to_vec(),
        Rc::clone(scope),
    )?))
}

pub fn eval(obj: &Rc<Object>, scope: &Rc<Scope>) -> Result<Rc<Object>, EvalErr> {
    let mut obj = obj.clone();
    let mut scope = scope.clone();
    loop {
        match obj.as_ref() {
            // resolve a symbol
            Object::Symbol(s) => {
                return if (s.starts_with("c") && s.ends_with("r"))
                    && (s.len() >= 3 && s.len() <= 6)
                    && (s[1..s.len() - 1].replace("a", "").replace("d", "")).is_empty()
                {
                    Ok(Rc::new(Object::Function(Function::Dynamic(s.clone()))))
                } else {
                    (scope.get(s)).ok_or_else(|| EvalErr::UnboundVariable(s.to_string()))
                };
            }
            // invoke a function
            Object::Pair(func, args) => {
                if let Ok(args) = list_to_vec(args) {
                    let result = invoke(func, args, &scope)?;
                    match result {
                        CallResult::Object(obj) => {
                            return Ok(obj);
                        }
                        CallResult::TailCall(obj2, scope2) => {
                            obj = obj2;
                            scope = scope2;
                        }
                    }
                } else {
                    return Err(EvalErr::ListRequired(args.to_string()));
                }
            }
            // other values evaluates to itself
            _ => return Ok(obj),
        }
    }
}

fn invoke(
    func: &Rc<Object>, args: Vec<Rc<Object>>, scope: &Rc<Scope>,
) -> Result<CallResult, EvalErr> {
    // special forms
    if let Object::Symbol(s) = func.as_ref() {
        if s == "quote" {
            return if args.len() != 1 {
                Err(EvalErr::WrongAgrsNum("quote".to_string(), 1, args.len()))
            } else {
                Ok(CallResult::Object(args[0].clone()))
            };
        } else if s == "if" {
            return fn_if(args, scope);
        } else if s == "let" {
            return fn_let(args, scope, false, false);
        } else if s == "let*" {
            return fn_let(args, scope, true, false);
        } else if s == "letrec" {
            return fn_let(args, scope, false, true);
        } else if s == "begin" {
            return fn_begin(args.as_slice(), scope);
        } else if s == "define" {
            fn_define(args, scope)?;
            return Ok(CallResult::Object(Rc::new(Object::Nil)));
        } else if s == "lambda" {
            return Ok(CallResult::Object(lambda(args, scope)?));
        } else if s == "and" {
            return logic_and(args, scope);
        } else if s == "or" {
            return logic_or(args, scope);
        } else if s == "cond" {
            return cond(args, scope);
        } else if s == "apply" {
            return fn_apply(args, scope);
        }
    }
    if let Object::Function(f) = eval(func, scope)?.as_ref() {
        f.call(eval_args(args, scope)?)
    } else {
        Err(EvalErr::IllegalObjectAsAFunction(func.to_string()))
    }
}
