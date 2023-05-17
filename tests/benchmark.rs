use scheme::errors::EvalErr;
use scheme::eval_expr;
use scheme::functions::Function;
use scheme::object::{Number, Object};
use scheme::scope::Scope;

use std::rc::Rc;
use std::time::Instant;

fn seq_sum(args: Vec<Rc<Object>>) -> Result<Rc<Object>, EvalErr> {
    if args.len() > 1 {
        return Err(EvalErr::TooManyArguments("seq_sum".to_string()));
    }
    let n = &args[0];
    if let &Object::Number(Number::Integer(n)) = n.as_ref() {
        let mut sum = 0;
        for i in 0..=n {
            sum += i;
        }
        Ok(Rc::new(Object::Number(Number::Integer(sum))))
    } else {
        Err(EvalErr::NumericArgsRequiredFor("seq_sum".to_string()))
    }
}

/// Testing of different loops<br>
/// The target function calculates sum of numbers from 0 to N using regular recursion and tail recursion approach.<br>
/// N is chosen so that recursive version does not overflow its stack<br>
/// Native Rust realization of the function used as a reference<br>
#[rustfmt::skip]
fn benchmark_loops() {
    const N: i32 = 500;
    const LOOP: i32 = 200;
    let test_fn = || -> i32 {
        // (0..=N).sum() is slower
        let mut sum = 0;
        for i in 0..=N {
            sum += i;
        }
        sum
    };
    let reference_sum: i32 = (0..=N).sum();
    let start = Instant::now();
    let scale = 1000;
    for _ in 0..LOOP * scale {
        assert_eq!(test_fn(), reference_sum);
    }
    let reference_time = start.elapsed().div_f64(scale as f64);
    println!(" {:<20} {:12?}", "Rust loop", reference_time);
    
    let scope = Rc::new(Scope::from_global());
    scope.bind("seq-sum", Rc::new(Function::from_pointer(seq_sum)));
    let scale = 100;
    let start = Instant::now();
    for _ in 0..LOOP * scale {
        let sum = eval_expr(&format!("(seq-sum {})", N), &scope).unwrap();
        assert_eq!(sum.to_string(), reference_sum.to_string());
    }
    let elapsed = start.elapsed().div_f64(scale as f64);
    println!(" {:<20} {:12?} ({:.2 }x)",
             "Rust implementation", elapsed, elapsed.as_secs_f64() / reference_time.as_secs_f64());
    
    let scope = Rc::new(Scope::from_global());
    let seq_sum = "
        (define (seq-sum n)
            (if (= 0 n)  0 (+ n (seq-sum (- n 1)))))";
    eval_expr(seq_sum, &scope).unwrap();
    let start = Instant::now();
    for _ in 0..LOOP {
        let sum = eval_expr(&format!("(seq-sum {})", N), &scope).unwrap();
        assert_eq!(sum.to_string(), reference_sum.to_string());
    }
    let elapsed = start.elapsed();
    println!(" {:<20} {:12?} ({:.0}x)",
            "Recursion", elapsed, elapsed.as_secs_f64() / reference_time.as_secs_f64());

    let scope = Rc::new(Scope::from_global());
    let seq_sum = "
        (define (seq-sum n)
            (define (seq-sum n acc)
                (if (= 0 n)  acc (seq-sum (- n 1) (+ acc n))))
            (seq-sum n 0))";
    eval_expr(seq_sum, &scope).unwrap();
    let start = Instant::now();
    for _ in 0..LOOP {
        let sum = eval_expr(&format!("(seq-sum {})", N), &scope).unwrap();
        assert_eq!(sum.to_string(), reference_sum.to_string());
    }
    let elapsed = start.elapsed();
    println!(" {:<20} {:12?} ({:.0}x)",
            "Tail recursion", elapsed, elapsed.as_secs_f64() / reference_time.as_secs_f64());
}

/// ```
/// assert_eq!(arr_to_string(&[1, 2, 3, 4, 5]), "(1 2 3 4 5)");
/// ```
fn arr_to_string(arr: &[i32]) -> String {
    let seq = (arr.iter())
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    format!("({})", seq)
}

/// Speed comparison of Rust zip+map, Scheme's built-in `map` function
/// and the same user-defined function `my-map` written in Scheme.
#[rustfmt::skip]
fn benchmark_map() {
    const LOOP: i32 = 1000;
    let arr1 = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut arr2 = arr1.clone();
    arr2.reverse();
    let result = [9, 16, 21, 24, 25, 24, 21, 16, 9];
    
    let test_fn = |arr1: &[i32], arr2: &[i32]| {
        return (arr1.iter().zip(arr2))
        .map(|(x, y)| x * y)
        .collect::<Vec<_>>();
    };

    let start = Instant::now();
    let scale = 1000;
    for _ in 0..LOOP * scale {
        let res = test_fn(&arr1, &arr2);
        assert_eq!(res, result);
    }
    let reference_time = start.elapsed().div_f64(scale as f64);
    println!(" {:<20} {:12?}", "Rust zip + map", reference_time);

    let start = Instant::now();
    let scope = Rc::new(Scope::from_global());
    let map1 = format!("(map * '{} '{})", arr_to_string(&arr1), arr_to_string(&arr2));
    let scale = 10;
    for _ in 0..LOOP * scale {
        let res = eval_expr(&map1, &scope).unwrap();
        assert_eq!(res.to_string(), arr_to_string(&result));
    }
    let elapsed1 = start.elapsed().div_f64(scale as f64);
    let x1 = elapsed1.as_secs_f64() / reference_time.as_secs_f64();
    println!(" {:<20} {:12?} ({:.0}x)", "Built-in map", elapsed1, x1);

    let my_map = "
        (define (map1 func list)
            (if (null? list)
                '()
                (cons (func (car list))
                    (map1 func (cdr list)))))
        (define (my-map func . lists)
            (if (null? (car lists))
                '()
                (cons (apply func (map1 car lists))
                    (apply my-map func (map1 cdr lists)))))";

    let start = Instant::now();
    let scope = Rc::new(Scope::from_global());
    eval_expr(my_map, &scope).unwrap();
    let map2 = format!("(my-map * '{} '{})", arr_to_string(&arr1), arr_to_string(&arr2));
    for _ in 0..LOOP {
        let res = eval_expr(&map2, &scope).unwrap();
        assert_eq!(res.to_string(), arr_to_string(&result));
    }
    let elapsed2 = start.elapsed();
    let x2 = elapsed2.as_secs_f64() / elapsed1.as_secs_f64();
    println!(" {:<20} {:12?} ({:.0}x * {:.0}x)", "Scheme map", elapsed2, x1, x2);
}

/// Counting number of primes in range 2..N in the simplest way.
#[rustfmt::skip]
fn behchmark_primes() {

    const N : i64 = 10_000;

    let test_fn = |n| {
        let mut acc = 0;
        for x in 2..n {
            for y in 2.. {
                if y * y > x {
                    acc += 1;
                    break;
                }
                if x % y == 0 {
                    break;
                }
            }
        }
        acc
    };

    let start = Instant::now();
    let scale = 1000;
    let result = test_fn(N);
    for _ in 0..scale {
        let res = test_fn(N);
        assert_eq!(res, result);
    }
    let reference_time = start.elapsed().div_f64(scale as f64);
    println!(" {:<20} {:12?}", "Rust impl", reference_time);

    let count_primes = "
        (define (prime? x y)
            (cond
                ((> (* y y) x) #t)
                ((= (modulo x y) 0) #f)
                (else (prime? x (+ y 1)))))
        (define (count-primes-acc n acc)
            (if (= n 1)
                acc
                (if (prime? n 2)
                    (count-primes-acc (- n 1) (+ acc 1))
                    (count-primes-acc (- n 1) acc))))
        (define (count-primes n)
            (count-primes-acc n 0))";

    let start = Instant::now();
    let scope = Rc::new(Scope::from_global());
    eval_expr(count_primes, &scope).unwrap();
    let scm_code: String = format!("(count-primes {})", N);
    {
        let res = eval_expr(&scm_code, &scope).unwrap();
        assert_eq!(res.to_string(), result.to_string());
    }
    let elapsed1 = start.elapsed();
    let x1 = elapsed1.as_secs_f64() / reference_time.as_secs_f64();
    println!(" {:<20} {:12?} ({:.0}x)", "Scheme impl", elapsed1, x1);
}

#[test]
fn run_bench() {
    println!("{:=^50}", "[ Loop benchmark ]");
    benchmark_loops();
    println!();

    println!("{:=^50}", "[ Map benchmark ]");
    benchmark_map();
    println!();

    println!("{:=^50}", "[ Count primes benchmark ]");
    behchmark_primes();
    println!();
}
