
use crate::object::*;
use crate::scope::*;
use crate::functions::*;
use std::rc::Rc;
use std::cell::RefCell;


/// Converts lists to Vec of references to its elements.
pub fn list_to_vec(mut obj : &Object) -> Result<Vec<Rc<Object>>, String> {
    let mut result = Vec::<Rc<Object>>::new();
    while !obj.is_nil() {
        match obj {
            Object::Pair(a, b) => {
                result.push(Rc::clone(a));
                obj = b.as_ref();
            },
            _ => return Err(format!("list required, but got {:?}", obj).to_string())
        }
    }
    Ok(result)
}

/// Ensures that given object is a list with length `n`
pub fn expect_args(args : &Object, func : &str, n : usize) -> Result<Vec<Rc<Object>>, String> {
    list_to_vec(args).and_then(|vec| {
        if vec.len() != n {
            Err(format!("Wrong number or arguments for '{}': {}", func, vec.len()))
        } else {
            Ok(vec)
        }
    })
}

/// Ensures that given object is a pair or returns an Err.
/// Since Rust doesn't support types for enum variants,
/// we are forced to use a tuple for the Ok value.
pub fn check_pair(obj : &Object) -> Result<(&Rc<Object>, &Rc<Object>), String> {
    match obj {
        Object::Pair(x, y) => Ok((x, y)),
        x => Err(format!("pair required but got {:?}", x).to_string())
    }
}

pub fn eval_args(args : &Object, scope : Rc<RefCell<Scope>>) -> Result<Rc<Object>, String> {
    match args {
        Object::Pair(a, b) => {
            eval(Rc::clone(a), Rc::clone(&scope))
                .and_then(|head| eval_args(b, scope)
                    .and_then(|tail| Ok(Rc::new(Object::Pair(head, tail)))))
        },
        _ => Ok(Rc::new(Object::Nil))
    }
}

fn quote(args : &Object) -> Result<Rc<Object>, String> {
    match args {
        Object::Pair(a, _) => Ok(Rc::clone(a)),
        _ => Err(format!("illegal argument for 'quote': {}", args))
    }
}

fn fn_if(args : &Object, scope : Rc<RefCell<Scope>>) -> Result<Rc<Object>, String> {
    expect_args(args, "if", 3).and_then(|vec| {
        let mut vec = vec.into_iter();
        eval(vec.next().unwrap(), Rc::clone(&scope)).and_then(|val| {
            if !val.is_true() {
                vec.next();
            }
            eval(vec.next().unwrap(), scope)
        })
    })
}

fn fn_let(args : &Object, scope : Rc<RefCell<Scope>>) -> Result<Rc<Object>, String> {
    list_to_vec(args).and_then(|vec| {
        if vec.len() < 2 {
            Err(format!("'let' need at least 2 arguments, {} found", vec.len()))
        } else {
            Ok(vec)
        }
    }).and_then(|let_args| {
        list_to_vec(let_args.get(0).unwrap()).and_then(|args| {
            let mut bindings = Vec::new();
            for arg in args {
                let result = list_to_vec(arg.as_ref()).and_then(|vec| {
                   return if vec.len() >= 2 {
                       let var = vec.get(0).unwrap().as_ref();
                       match var {
                           Object::Symbol(s) => {
                               eval(Rc::clone(vec.get(1).unwrap()), Rc::clone(&scope))
                                   .map(|obj| bindings.push((s.clone(), obj)))
                           },
                           _ => Err(format!("let: need a symbol for binding name, got {}", var).to_string())
                       }
                   } else {
                       Err(format!("let: need a list length 2 for bindings, got {}", arg).to_string())
                   }
                });
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
            }
            fn_begin_vec(let_args.into_iter().skip(1),
                         Rc::new(RefCell::new(Scope::new(bindings.as_slice(), Some(scope)))))
        })
    })
}

fn fn_begin_vec(args : impl ExactSizeIterator<Item=Rc<Object>>, scope : Rc<RefCell<Scope>>)
                    -> Result<Rc<Object>, String> {
    if args.len() == 0 {
        return Err("'begin' need at least 1 argument".to_string());
    }
    let mut result : Option<Rc<Object>> = None;
    for arg in args {
        match eval(arg, Rc::clone(&scope)) {
            Err(e) => return Err(e),
            Ok(val) => result = Some(val)
        }
    }
    return Ok(result.unwrap());
}

pub fn fn_begin(args : &Object, scope : Rc<RefCell<Scope>>) -> Result<Rc<Object>, String> {
    list_to_vec(args).and_then(|args| fn_begin_vec(args.into_iter(), scope))
}

/// There are two forms of define:
///
/// `(define id expr)` and `(define (id args...) expr...)`
///
/// Both syntax handled by this function. Defined value added to a current scope
pub fn fn_define(args : &Object, scope : Rc<RefCell<Scope>>) -> Result<(), String> {
    match args {
        Object::Pair(head, expr) => {
            match head.as_ref() {
                // (define x expr)
                Object::Symbol(s) => {
                    match expr.as_ref() {
                        Object::Pair(value, tail) => {
                            if tail.as_ref() != &Object::Nil {
                                Err("extra argument for 'define'".to_string())
                            } else {
                                eval(Rc::clone(value), Rc::clone(&scope))
                                    .map(|obj| scope.as_ref().borrow_mut().bind(s.to_string(), obj))
                            }
                        },
                        _ => Err(format!("proper list required for 'define': {}", args).to_string())
                    }
                },
                // (define (name args) body)
                Object::Pair(name, args) => {
                    match name.as_ref() {
                        Object::Symbol(s) => {
                            let name = s.to_string();
                            let obj = Rc::new(Object::Function(Function::Object {
                                name: name.clone(),
                                args: Rc::clone(args),
                                body: Rc::clone(expr),
                                scope: Rc::clone(&scope)
                            }));
                            scope.as_ref().borrow_mut().bind(name, obj);
                            Ok(())
                        },
                        _ => Err(format!("expected symbol as a function name, found {}", name).to_string())
                    }
                },
                x => Err(format!("proper list or symbol need for 'define', found: {}", x).to_string())
            }
        },
        _ => Err(format!("a list expected for 'define' arguments, found {}", args))
    }
}

pub fn eval(obj : Rc<Object>, scope : Rc<RefCell<Scope>>) -> Result<Rc<Object>, String> {
    match obj.as_ref() {
        // resolve a symbol
        Object::Symbol(s) => {
            Ok(scope.borrow().get(s).unwrap_or(obj))
        },
        // invoke a function
        Object::Pair(func, args) => {
            eval(Rc::clone(func), Rc::clone(&scope)).and_then(|rc| {
                let new_scope = Rc::new(RefCell::new(Scope::new(&[], Some(Rc::clone(&scope)))));
                match rc.as_ref() {
                    Object::Symbol(s) => {
                        return if s == "quote" {
                            quote(args.as_ref())
                        } else if s == "if" {
                            fn_if(args.as_ref(), new_scope)
                        } else if s == "let" {
                            fn_let(args.as_ref(), new_scope)
                        } else if s == "begin" {
                            fn_begin(args.as_ref(), new_scope)
                        } else if s == "define" {
                            fn_define(args.as_ref(), scope).map(|_| Rc::new(Object::Nil))
                        } else {
                            Err(format!("expected a function, found unbound symbol '{}'", s))
                        }
                    },
                    Object::Function(f) => {
                        eval_args(args, scope).and_then(|args| f.call(args))
                    },
                    _ => Err(format!("Illegal object used as a function: {:?}", func))
                }
            })
        }
        // other values evaluates to itself
        _ => Ok(obj)
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use crate::parser::parse_expression;

    fn assert_eval(expr : &str, expected: &str) {
        let scope = get_global_scope();
        let obj = parse_expression(expr).unwrap().pop().unwrap();
        eval(Rc::new(obj), Rc::new(RefCell::new(scope)))
            .map(|obj| assert_eq!(format!("{}", obj), expected))
            .unwrap_or_else(|err| panic!(err));
    }

    fn expect_err(expr: &str) {
        let scope = get_global_scope();
        let obj = parse_expression(expr).unwrap().pop().unwrap();
        eval(Rc::new(obj), Rc::new(RefCell::new(scope)))
            .expect_err(format!("error expected for {}", expr).as_str());
    }

    #[test]
    fn eval_test() {
        assert_eval("'1", "1");
        assert_eval("(car '(1 . 2))", "1");
        assert_eval("(cdr '(1 . 2))", "2");
        assert_eval("(car (cdr '(1 2 3)))", "2");
        assert_eval("(length '(1 2 3))", "3");
        assert_eval("(cons 1 2)", "(1 . 2)");
        assert_eval("(list 1 2 3)", "(1 2 3)");

        assert_eval("(if #t 1 2)", "1");
        assert_eval("(if #f 1 2)", "2");

        assert_eval("(begin 1 2 3)", "3");
        assert_eval("(let ((x 2)) x)", "2");
        assert_eval("(let ((x car) (y '(1 2 3))) (x y))", "1");
        assert_eval("(begin (define x 5) (cons (begin (define x 2) x) x))", "(2 . 5)");

        assert_eval("(begin (define (x)) x)", "<function>");
        assert_eval("(begin (define (x a) (car a)) (x '(5 6)))", "5");
        assert_eval("(begin (define (tail a . b) b) (tail 1 2 3))", "(2 3)");

        expect_err("(begin (define (f a b c)) (f 1))");
        expect_err("(begin (define (f a b c)) (f 1 2 3 4))");
    }

}
