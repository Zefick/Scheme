use std::rc::Rc;

use scheme::errors::EvalErr;
use scheme::eval::eval;
use scheme::parser::parse_expression;
use scheme::scope::Scope;
use scheme::{eval_expr, eval_file};

fn assert_eval(expr: &str, expected: &str) {
    assert_eval_with_scope(&Rc::new(Scope::from_global()), expr, expected);
}

fn assert_eval_with_scope(scope: &Rc<Scope>, expr: &str, expected: &str) {
    let obj = parse_expression(expr).unwrap().pop().unwrap();
    match eval(&Rc::new(obj), &scope) {
        Ok(obj) => assert_eq!(format!("{}", obj), expected),
        Err(err) => panic!("{}", err),
    }
}

fn expect_err(expr: &str, expected: EvalErr) {
    let scope = Scope::from_global();
    let obj = parse_expression(expr).unwrap().pop().unwrap();
    let result = eval(&Rc::new(obj), &Rc::new(scope));
    match result {
        Ok(_) => panic!(
            "expression {} expected to evaluate with the error\n\"{}\"",
            expr, expected
        ),
        Err(err) => {
            if err != expected {
                panic!(
                    "expression \"{}\" expected to evaluate with the error\n\"{}\" but the actual error is \n\"{}\"",
                    expr, expected, err
                );
            }
        }
    }
}

#[test]
#[rustfmt::skip]
fn core_functions() {
    assert_eval("(begin 1 2 3)", "3");
    assert_eval("'1", "1");
    assert_eval("'''foo", "(quote (quote foo))");
    expect_err("(list a)", EvalErr::UnboundVariable("a".to_string()));
    expect_err("(begin 1 . 2)", EvalErr::ListRequired("(1 . 2)".to_string()));
    expect_err("(quote 1 2)", EvalErr::WrongAgrsNum("quote".to_string(), 1, 2));
}

#[test]
#[rustfmt::skip]
fn lists_and_pairs() {
    assert_eval("(car '(1 . 2))", "1");
    assert_eval("(cdr '(1 . 2))", "2");
    assert_eval("(car (cdr '(1 2 3)))", "2");
    assert_eval("(cadr '(1 2 3))", "2");
    assert_eval("(cadar '((1 2) 3))", "2");
    assert_eval("(cddr '(1 2 3))", "(3)");
    assert_eval("(caaaar '((((1 2 3))) 4))", "1");
    expect_err ("(caaaaar '())", EvalErr::UnboundVariable("caaaaar".to_string()));
    assert_eval("(length '(1 2 3))", "3");
    assert_eval("(cons 1 2)", "(1 . 2)");
    assert_eval("(cons 1 '(2 3))", "(1 2 3)");
    assert_eval("(list 1 2 3)", "(1 2 3)");
    expect_err("(car 5)", EvalErr::PairRequired("5".to_string()));
}

#[test]
#[rustfmt::skip]
fn let_and_define() {
    assert_eval("(begin (define x 5) (cons (begin (define x 2) x) x))", "(2 . 2)");
    assert_eval("(begin (define (x a) (car a)) (x '(5 6)))", "5");
    assert_eval("(begin (define (tail a . b) b) (tail 1 2 3))", "(2 3)");
    expect_err("(define 5)", EvalErr::WrongDefineArgument("5".to_string()));
    expect_err("(define (5))", EvalErr::ExpectedSymbolForFunctionName("5".to_string()));
    expect_err("(define (f x))", EvalErr::EmptyFunctionBody());
}

#[test]
#[rustfmt::skip]
fn lambda() {
    // lambda functions
    assert_eval("((lambda x x) 1 2 3)", "(1 2 3)");
    assert_eval("((lambda (x . y) y) 1 2 3)", "(2 3)");
    assert_eval("(begin (define f (lambda (x) x)) (f 'foo))", "foo");
    expect_err("(lambda (1) a)", EvalErr::ExpectedSymbolForArgument("1".to_string()));
    expect_err("(lambda (x x) x)", EvalErr::ArgumentDuplication("x".to_string()));
    expect_err("((lambda (a b) a) 1)", EvalErr::TooFewArguments("#<lambda>".to_string()));
    expect_err("((lambda (a b) a) 1 2 3)", EvalErr::TooManyArguments("#<lambda>".to_string()));
    expect_err("(5)", EvalErr::IllegalObjectAsAFunction("5".to_string()));
}

#[test]
#[rustfmt::skip]
fn logic_functions() {
    assert_eval("(or 5 foo #f)", "5");
    expect_err("(or #f foo)", EvalErr::UnboundVariable("foo".to_string()));
    assert_eval("(and 5 'foo 42)", "42");
    assert_eval("(list (boolean? #f) (boolean? 5))", "(#t #f)");
    assert_eval("(list (null? #t) (null? '(5)) (null? '()) (null? (cdr '(5))))", "(#f #f #t #t)");
    assert_eval("(list (pair? '(1 2)) (pair? 5))", "(#t #f)");
    assert_eval("(list (list? '(1 2)) (list? 5) (list? '(1 . 2)))", "(#t #f #f)");
    assert_eval("(list (not #f) (not 5))", "(#t #f)");
}

#[test]
#[rustfmt::skip]
fn if_cond() {
    assert_eval("(if #t 1 2)", "1");
    assert_eval("(if #f 1 2)", "2");
    assert_eval("(cond (#f 42) ('foo))", "foo");
    assert_eval("(cond (#f 42) (5 'foo))", "foo");
    assert_eval("(cond (#f 42))", "#<undef>");
    assert_eval("(cond (#f 42) (else))", "#<undef>");
    assert_eval("(cond (#f 42) (else 1 2))", "2");
    assert_eval("(cond (#f 42) (#t 1 2))", "2");
    expect_err("(cond)", EvalErr::CondNeedsClause());
    expect_err("(cond ())", EvalErr::CondEmptyClause());
}

#[test]
#[rustfmt::skip]
fn test_math() {
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
    assert_eval("(integer? (/ (+ 3 5) (* 1 2)))", "#t");
    assert_eval("(integer? (/ (+ 3 4) (* 1 2)))", "#f");
    assert_eval("(list (quotient 13 4) (quotient -13 4) (quotient 13 -4) (quotient -13 -4))", "(3 -3 -3 3)");
    assert_eval("(list (remainder 13 4) (remainder -13 4) (remainder 13 -4) (remainder -13 -4))", "(1 -1 1 -1)");
    assert_eval("(list (modulo 13 4) (modulo -13 4) (modulo 13 -4) (modulo -13 -4))", "(1 3 -3 -1)");
    expect_err("(= 1 foo)", EvalErr::UnboundVariable("foo".to_string()));
    expect_err("(+ 1 'foo)", EvalErr::NumericArgsRequiredFor("+".to_string()));
    expect_err("(/ 1 2 0)", EvalErr::DivisionByZero());
    expect_err("(modulo 13.5 4)", EvalErr::IntegerArgsRequiredFor("modulo".to_string()));
}

#[test]
#[rustfmt::skip]
fn apply_and_map() {
    assert_eval("(apply list '(1 2 3))", "(1 2 3)");
    assert_eval("(apply list 1 2 '(3 4))", "(1 2 3 4)");
    assert_eval("(apply list 1 (+ 1 1) '(3 (+ 2 2)))", "(1 2 3 (+ 2 2))");
    assert_eval("(let ((foo (lambda (x) (+ x 10)))) (apply foo '(0)))", "10");
    expect_err("(apply + 1 2 3)", EvalErr::ApplyNeedsProperList("3".to_string()));
    expect_err("(apply +)", EvalErr::NeedAtLeastArgs("apply".to_string(), 2, 1));
    assert_eval("(map list '(1 2 3))", "((1) (2) (3))");
    assert_eval("(map list '(1 2 (+ 1 2)))", "((1) (2) ((+ 1 2)))");
    assert_eval("(map list '(1 2 3) '(4 5 6))", "((1 4) (2 5) (3 6))");
    expect_err("(map + '(1 2) '(4 5 6))", EvalErr::UnequalMapLists());
    expect_err("(map +)", EvalErr::NeedAtLeastArgs("map".to_string(), 2, 1));
    assert_eval("(map (lambda (x, y) (+ x y)) '())", "()");
}

#[test]
#[rustfmt::skip]
fn equalities() {
    assert_eval("(list (eqv? '() '()) (eqv? '(a) '(a)) (eqv? '(()) '(())))", "(#t #f #f)");
    assert_eval("(list (eqv? #t #t) (eqv? #t #f) (eqv? #t 42))", "(#t #f #f)");
    assert_eval("(list (eqv? 'a 'a) (eqv? 'a 'b))", "(#t #f)");
    assert_eval("(eqv? (lambda () 1) (lambda () 1))", "#f");
    assert_eval("(let ((p (lambda (x) x))) (eqv? p p))", "#t");
    assert_eval("(let ((a '(a)) (b '(a))) (list (eqv? a a) (eqv? a b)))", "(#t #f)");
    assert_eval("(let ((a '(a b))) (eqv? (cdr a) (cdr a)))", "#t");
    assert_eval("(let ((a '(a b))) (eqv? (cdr a) '(b)))", "#f");
    assert_eval("(eqv? car car)", "#t");
    assert_eval("(eqv? cdadar cdadar)", "#t");

    assert_eval("(list (eqv? 2 2) (eqv? 2 3) (eqv? 2 2.0))", "(#t #f #t)");
    assert_eval("(list (eq? 2 2) (eq? 2 3) (eq? 2 2.0))", "(#t #f #f)");

    assert_eval("(equal? '(a b (c)) '(a b (c)))", "#t");
    assert_eval("(equal? '(a b (c)) '(a b c))", "#f");
    assert_eval("(equal? '(a b (c)) '(a b))", "#f");
    assert_eval("(equal? '(2) '(2.0))", "#t");
}

#[test]
#[rustfmt::skip]
fn prelude() {
    let scope = &Rc::new(Scope::from_global());
    assert!(eval_file("prelude.scm", scope).is_ok());
    assert_eval_with_scope(scope, "(foldr cons '() '(1 2 3))", "(1 2 3)");
    assert_eval_with_scope(scope, "(foldl cons '() '(1 2 3))", "(((() . 1) . 2) . 3)");
    assert_eval_with_scope(scope, "(append '(1 2) '(3 4))", "(1 2 3 4)");
    assert_eval_with_scope(scope, "(reverse '(1 2 3 4))", "(4 3 2 1)");
}

#[test]
#[rustfmt::skip]
fn test_let() {
    assert_eval("(let ((x 2)) x)", "2");
    assert_eval("(let ((x car) (y '(1 2 3))) (x y))", "1");
    
    assert_eval("(let ((x 11)) (let ((x 22) (y x)) y))", "11");
    assert_eval("(let ((x 11)) (let* ((x 22) (y x)) y))", "22");
    
    expect_err("(let ((z 22) (y z)) y)", EvalErr::UnboundVariable("z".to_string()));
    assert_eval("(let* ((z 22) (y z)) y)", "22");

    let body = "((fun (lambda () fun))) (fun)";
    expect_err(&format!("(let {})", body), EvalErr::UnboundVariable("fun".to_string()));
    assert_eval(&format!("(letrec {})", body), "<function>");
    
    assert_eval("(let ((x 2)) (map (lambda (y) (+ x y)) '(1 2 3)))", "(3 4 5)");
    assert_eval("(let ((f (lambda (y) (+ y 2)))) (map f '(1 2 3)))", "(3 4 5)");
    assert_eval("(let ((x (lambda () '(1 2 3)))) (map + (x) (x)))", "(2 4 6)");

    assert_eval("
        (letrec
          ((fib (lambda (n) (if (< n 2) 1 (+ (fib1 n) (fib2 n)))))
           (fib1 (lambda (n) (fib (- n 1))))
           (fib2 (lambda (n) (fib (- n 2))))
           (x (fib 10)))
          x)", "89");
}

#[test]
#[rustfmt::skip]
/// Verifies that tail calls are working properly.
/// That is, tail recursion does not lead to stack overflow.
fn test_tail_call() {
    let scope = &Rc::new(Scope::from_global());
    
    // sum of 10000 consecutive integers
    let seq_sum = "
        (define (seq-sum n)
          (define (seq-sum n acc)
            (if (= 0 n)  acc (seq-sum (- n 1) (+ acc n))))
          (seq-sum n 0))";
    eval_expr(seq_sum, scope).unwrap();
    assert_eval_with_scope(scope, "(seq-sum 10000)", "50005000");

    // mutual recursion
    let is_odd = "
        (define (odd? n)
            (if (= n 0) #f (even? (- n 1))))";
    let is_even = "
        (define (even? n)
            (if (= n 0) #t (odd? (- n 1))))";
    eval_expr(is_odd, scope).unwrap();
    eval_expr(is_even, scope).unwrap();
    assert_eval_with_scope(scope, "(map even? '(10500 9999))", "(#t #f)");

    // tail recursion with 'apply'
    let seq_sum = "
        (define (seq-sum n)
          (define (seq-sum n acc)
            (if (= 0 n) acc (apply seq-sum (list (- n 1) (+ acc n)))))
          (seq-sum n 0))";
    eval_expr(seq_sum, scope).unwrap();
    assert_eval_with_scope(scope, "(seq-sum 10000)", "50005000");
}
