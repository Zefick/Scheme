# Scheme
Implementation of Scheme interpreter in Rust

This is an educational project that I started to practice developing in Rust.
The project's goal is to implement core features of the Scheme language.
The stability of implementation leans on much number of tests that cover as any features as possible.

Currently interpreter supports:

* Data types: symbols, strings, numbers, dotted pairs. Boolean values presented as symbols `#t` and `#f`.

* Base special forms and functions: `define`, `if`, `cond`, `quote`, `begin`.
```
''(42)                                      => (quote 42)
(begin 1 2 (+ 1 2)))                        => 3
(define x 5)
(let ((y 6)) (* x y))                       => 30
(if (> 1 2) (not-evaluated) #t)             => #t
```

* Local bindings with `let` including such forms as `let*` and `letrec`
```
(let ((x 11)) (let ((x 22) (y x)) y))       => 11
(let ((x 11)) (let* ((x 22) (y x)) y))      => 22

(letrec
  ((fib (lambda (n) (if (< n 2)
        1
        (+ (fib (- n 1))
           (fib (- n 2))))))
   (x (fib 10)))
  x)                                        => 89
```
> **Note**:<br>
My `letrec` actually works as the `letrec*` from Scheme.<br>
The Racket abandoned `letrec*` and redefined its `letrec` to make it work as old `letrec*`.
I made the same in order not to implement two separate form.

* Functions as first class citizens, lexical scoping, lambdas and related functions: `apply`, `map`.
```
(apply + 1 2 3 4)                           => 10
(map (lambda (x) (* x x)) '(1 2 3))         => (1 4 9)
(define x 5)
(define y 6)
(let ((y 10)) (* x y))                      => 50
(* x y)                                     => 30
```

* Functions for working with pairs and lists: `cons`, `list`, `car`, `cdr`, `cadr` and other of this kind.
```
(cons 1 2)                                  => (1 . 2)
(list 1 2 3)                                => (1 2 3)
```
* Simple math operations: `+`, `-`, `*`, `/`, `=`, `>`, `<`.
```
(list (+) (+ 1) (+ 1 3))                    => (0 1 4)
(list (-) (- 1) (- 1 3))                    => (0 -1 -2)
(list (*) (* 2) (* 2 3))                    => (1 2 6)
(list (/) (/ 2) (/ 2 3))                    => (1 0.5 0.6666...)
```

* Lazy logic operations: `and`, `or`; and predicates `number?`, `boolean?`, `list?`, `pair?`, `null?`.
```
(define test-arr '((1 2) 3 #t ()))
(map number?   test-arr)                    => (#f #t #f #f)
(map boolean?  test-arr)                    => (#f #f #t #f)
(map list?     test-arr)                    => (#t #f #f #t)
(map pair?     test-arr)                    => (#t #f #f #f)
(map null?     test-arr)                    => (#f #f #f #t)
```

* Comparison functions: `=`, `>`, `<` for numbers and `eq?`, `eqv?`, `equals?` for other objects and lists.
```d
(= (+ 1 2) 3)                               => #t
(list (> 1 2) (> 3 2 1))                    => (#f #t)
(list (= 1 2) (= 1 1.0))                    => (#f #t)
(eq? 1 1.0)                                 => #f
(eqv? 1 1.0)                                => #t
(eqv? '(1 2 3) '(1 2 3))                    => #f
(equal? '(1 2 3) '(1 2 3))                  => #t
```

* Recognition and optimization of tail calls.
  * Calls in tail positions in such forms and functions as `let`, `begin`, `apply`, `if`, `and`, `or` and in user-defined functions
optimized so that their repetitive recursive calls do not lead to stack growth.

```
(define (seq-sum n)
  (define (seq_sum n acc)
    (if (= 0 n)  acc (seq-sum (- n 1) (+ acc n))))
  (seq-sum n 0))
  
(seq-sum 10000)                             => 50005000
```

##### The features which still not implemented and maybe will not but need to be mentioned:

###### Full Scheme math

The specs of Scheme language implies using several numeric types: integer, real, rational fractions, complex.
Besides, real numbers may be exact or inexact, integer and rational numbers may have very big length.
All numeric functions must be able to operate with number of any type.
My implementation provides only two types of numbers: integer and float represented as `i32` and `f64` respectively.
There is no such categories as exact and inexact numbers and big integers.
Full realization of math from specs need a lot of effort by itself. This is not a goal of the project.

###### Memory management

Memory management leans on Rust's smart pointers. So if there is no cyclic structures then memory should free automatically.
There is no fully functional GC.

###### call-with-current-continuation
I'll try to implement it as I figure out what it is and how useful it is.

###### Quasi-quoting
A feature that looks easy to implement, but not so useful for common tasks.
Therefore, I prefer not to implement this, at least at the current stage.

###### Miscellaneous

If there are other parts of the language that are not listed here it means that I forgot, don't know about them, they are not so important or all of the above.
