use crate::eval::*;
use crate::parser::parse_expression;
use crate::scope::get_global_scope;
use std::cell::RefCell;
use std::rc::Rc;

fn assert_eval(expr: &str, expected: &str) {
    let scope = get_global_scope();
    let obj = parse_expression(expr).unwrap().pop().unwrap();
    eval(&Rc::new(obj), &Rc::new(RefCell::new(scope)))
        .map(|obj| assert_eq!(format!("{}", obj), expected))
        .unwrap_or_else(|err| panic!(err));
}

fn expect_err(expr: &str) {
    let scope = get_global_scope();
    let obj = parse_expression(expr).unwrap().pop().unwrap();
    eval(&Rc::new(obj), &Rc::new(RefCell::new(scope)))
        .expect_err(format!("error expected for {}", expr).as_str());
}

#[test]
#[rustfmt::skip]
fn eval_test() {

    // core functions and special forms
    expect_err("(list a)");                   // unbound variable
    expect_err("(begin 1 . 2)");              // not a proper list
    assert_eval("(begin 1 2 3)", "3");
    assert_eval("'1", "1");
    assert_eval("'''foo", "(quote (quote foo))");

    // list and pairs functions
    assert_eval("(car '(1 . 2))", "1");
    assert_eval("(cdr '(1 . 2))", "2");
    assert_eval("(car (cdr '(1 2 3)))", "2");
    assert_eval("(length '(1 2 3))", "3");
    assert_eval("(cons 1 2)", "(1 . 2)");
    assert_eval("(list 1 2 3)", "(1 2 3)");

    // let, define
    assert_eval("(let ((x 2)) x)", "2");
    assert_eval("(let ((x car) (y '(1 2 3))) (x y))", "1");
    assert_eval("(begin (define x 5) (cons (begin (define x 2) x) x))", "(2 . 5)");
    assert_eval("(begin (define (x)) x)", "<function>");
    assert_eval("(begin (define (x a) (car a)) (x '(5 6)))", "5");
    assert_eval("(begin (define (tail a . b) b) (tail 1 2 3))", "(2 3)");

    // lambda functions
    assert_eval("((lambda x x) 1 2 3)", "(1 2 3)");
    assert_eval("((lambda (x . y) y) 1 2 3)", "(2 3)");
    assert_eval("(begin (define f (lambda (x) x)) (f 'foo))", "foo");
    expect_err("(lambda (1) a)");             // wrong argument type
    expect_err("(lambda (x x) x)");           // duplication of argument id
    expect_err("((lambda (a b) a) 1)");       // too few arguments given
    expect_err("((lambda (a b) a) 1 2 3)");   // too many arguments given

    // logic functions
    assert_eval("(if #t 1 2)", "1");
    assert_eval("(if #f 1 2)", "2");
    assert_eval("(or 5 foo #f)", "5");
    expect_err("(or #f foo)");                // unbound variable
    assert_eval("(and 5 'foo 42)", "42");
    assert_eval("(list (boolean? #f) (boolean? 5))", "(#t #f)");
    assert_eval("(list (pair? '(1 2)) (pair? 5))", "(#t #f)");
    assert_eval("(list (list? '(1 2)) (list? 5) (list? '(1 . 2)))", "(#t #f #f)");
    assert_eval("(list (not #f) (not 5))", "(#t #f)");
    assert_eval("(cond (#f 42) ('foo))", "foo");
    assert_eval("(cond (#f 42) (5 'foo))", "foo");
    assert_eval("(cond (#f 42))", "#<undef>");
    assert_eval("(cond (#f 42) (else))", "#<undef>");
    assert_eval("(cond (#f 42) (else 1 2))", "2");

    // arithmetic functions
    assert_eval("(list (number? 1) (number? 'foo))", "(#t #f)");
    assert_eval("(list (integer? 1) (integer? 1.0))", "(#t #f)");
    assert_eval("(list (real? 1) (real? 1.0))", "(#t #t)");
    assert_eval("(list (= 1 1 2) (= 1 1.0))", "(#f #t)");
    assert_eval("(list (< 1 2.0 3) (< 1 3 2))", "(#t #f)");
    assert_eval("(list (> 5 3 0) (> 5 1.0 3))", "(#t #f)");
    assert_eval("(list (+) (+ 1) (+ 1 2 3) (+ 1 2 3.5))", "(0 1 6 6.5)");
    assert_eval("(list (-) (- 1) (- 1 2.5) (- 1 2 3))", "(0 -1 -1.5 -4)");
    assert_eval("(list (*) (* 2) (* 1 2 3.5))", "(1 2 7)");
    assert_eval("(list (/) (/ 2) (/ 2 1))", "(1 0.5 2)");
    assert_eval("(list (integer? (/ 2 1)) (integer? (/ 1 2)))", "(#t #f)");
    expect_err("(= 1 foo)");                 // wrong arguments
    expect_err("(+ 1 'foo)");                // wrong type
    expect_err("(/1 2 0)");                  // division by 0
}
