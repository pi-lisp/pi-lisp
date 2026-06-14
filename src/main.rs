mod builtins;
mod env;
mod eval;
mod expr;
mod macros;
mod reader;

use env::Env;
use eval::eval;
use reader::parse_all;

/// Parses and evaluates each top-level expression in `src`, printing results.
fn run(src: &str, env: &Env) {
    match parse_all(src) {
        Ok(exprs) => {
            for e in exprs {
                match eval(&e, env) {
                    Ok(result) => println!("{} => {:?}", src, result),
                    Err(err) => println!("{} => Error: {}", src, err),
                }
            }
        }
        Err(err) => println!("{} => Parse error: {}", src, err),
    }
}

fn main() {
    let env = builtins::global_env();

    let exprs = vec![
        "(define square (lambda (x) (* x x)))",
        "(square 5)",
        "(define fact (lambda (n) (if (< n 1) 1 (* n (fact (- n 1))))))",
        "(fact 10)",
        "(let ((a 3) (b 4)) (+ (* a a) (* b b)))",
        // macro: unless
        "(defmacro unless (cond then) (list 'if (list 'not cond) then 0))",
        "(unless 0 (+ 1 2))", // cond is 0 (false) -> evaluates then -> 3
        "(unless 1 (+ 1 2))", // cond is 1 (true)  -> 0
        // macro: my-or
        "(defmacro my-or (a b) (list 'if a a b))",
        "(my-or 0 42)",
        "(my-or 7 42)",
        // quasiquote / unquote
        "(define x 10)",
        "(quasiquote (a b (unquote x)))",
        "(define lst (list 1 2 3))",
        "(quasiquote (start (unquote-splicing lst) end))",
        // quote sugar
        "'(1 2 3)",
        "(car '(1 2 3))",
        "(cdr '(1 2 3))",

        // interval / path examples (cubical-flavored)
        "i0",
        "i1",
        // a path that linearly interpolates between 1 and 5
        "(define interp (path (i) (+ (* (- 1 i) 1) (* i 5))))",
        "(papply interp i0)",   // => 1
        "(papply interp i1)",   // => 5
        "(papply interp 0.5)",  // => 3
        // refl: the constant path
        "(define rp (refl 42))",
        "(papply rp i0)", // => 42
        "(papply rp i1)", // => 42
        "(papply rp 0.3)", // => 42
    ];

    for src in exprs {
        run(src, &env);
    }
}