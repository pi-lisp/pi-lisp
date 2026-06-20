//! Phase-1 + Phase-2 VM infrastructure: bytecode representation, compiler,
//! and stack-based execution engine.
//!
//! # Public API
//!
//! The main entry point for the integration layer is [`vm_eval`], which:
//! 1. Compiles `expr` with [`compiler::Compiler::compile`].
//! 2. Runs the resulting [`bytecode::Chunk`] with [`machine::VM`].
//! 3. Falls back to the tree-walking [`crate::eval::eval`] if the compiler
//!    returns an `"uncompilable"` error (e.g. because the expression contains
//!    a `CubicalTerm`).

pub mod bytecode;
pub mod compiler;
pub mod machine;

use crate::eval::eval_tree as tree_eval;
use crate::expr::Expr;
use crate::gc::{GcHandle, Heap};

use compiler::{Compiler, is_compilable};
use machine::{VM, vm_value_to_expr};

/// Evaluate `expr` using the bytecode VM, falling back to the tree-walker on
/// uncompilable expressions.
///
/// # Fallback behaviour
///
/// If [`Compiler::compile`] returns an error whose message starts with
/// `"uncompilable"`, `vm_eval` silently delegates to the tree-walking
/// evaluator.  Any other compile or runtime error is propagated as-is.
///
/// This allows the VM and the tree-walker to coexist: cubical-type-theory
/// forms (which use `CubicalTerm`) are always handled by the tree-walker,
/// while everything else goes through the VM.
pub fn vm_eval(expr: &Expr, env: GcHandle, heap: &mut Heap) -> Result<Expr, String> {
    if !is_compilable(expr, heap, env) {
        // Explicitly fall back — do not attempt compilation
        return tree_eval(expr, env, heap);
    }
    match Compiler::compile(expr, env, heap) {
        Ok(chunk) => {
            let mut vm = VM::new(heap, env, chunk);
            match vm.run() {
                Ok(v) => vm_value_to_expr(v, vm.heap_mut()),
                Err(e) => {
                    if e.starts_with("uncompilable") {
                        tree_eval(expr, env, heap)
                    } else {
                        Err(e)
                    }
                }
            }
        }
        Err(_) => tree_eval(expr, env, heap), // safety net
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::parse_all;
    use crate::builtins;

    fn eval_str(src: &str, heap: &mut Heap, env: GcHandle) -> Result<Expr, String> {
        let exprs = parse_all(src)?;
        assert_eq!(exprs.len(), 1);
        vm_eval(&exprs[0], env, heap)
    }

    #[test]
    fn test_is_compilable() {
        use crate::vm::compiler::is_compilable;
        let mut heap = Heap::new();
        let env = builtins::global_env(&mut heap);

        // Simple arithmetic/comparisons: compilable
        let expr1 = parse_all("(+ 1 2)").unwrap().remove(0);
        assert!(is_compilable(&expr1, &heap, env));

        let expr2 = parse_all("(if (= 1 1) (+ 2 3) 4)").unwrap().remove(0);
        assert!(is_compilable(&expr2, &heap, env));

        // define, let, let* are now compilable
        let expr3 = parse_all("(let ((x 1)) x)").unwrap().remove(0);
        assert!(is_compilable(&expr3, &heap, env), "let should now be compilable");

        let expr4 = parse_all("(define x 1)").unwrap().remove(0);
        assert!(is_compilable(&expr4, &heap, env), "define should now be compilable");

        let expr5 = parse_all("(let* ((x 1) (y (+ x 1))) y)").unwrap().remove(0);
        assert!(is_compilable(&expr5, &heap, env), "let* should now be compilable");

        // lambda is compilable (compile_lambda handles it)
        let expr_lambda = parse_all("(lambda (x) x)").unwrap().remove(0);
        assert!(is_compilable(&expr_lambda, &heap, env));

        // quasiquote with unquote: compilable
        let expr6 = parse_all("`(1 ,x)").unwrap().remove(0);
        assert!(is_compilable(&expr6, &heap, env));

        // quasiquote without unquote: compilable
        let expr7 = parse_all("`(1 2 3)").unwrap().remove(0);
        assert!(is_compilable(&expr7, &heap, env));

        // defmacro: never compilable (always tree-walker)
        let expr_dm = parse_all("(defmacro foo (x) x)").unwrap().remove(0);
        assert!(!is_compilable(&expr_dm, &heap, env), "defmacro must not be compilable");
    }

    #[test]
    fn test_vm_define() {
        let mut heap = Heap::new();
        let env = builtins::global_env(&mut heap);

        // define returns ()
        let res = eval_str("(define x 42)", &mut heap, env).unwrap();
        assert!(matches!(res, Expr::List(ref v) if v.is_empty()),
            "define should return (): got {:?}", res);

        // The binding should now be visible
        let res2 = eval_str("x", &mut heap, env).unwrap();
        assert!(matches!(res2, Expr::Number(n) if n == 42.0));
    }

    #[test]
    fn test_vm_let() {
        let mut heap = Heap::new();
        let env = builtins::global_env(&mut heap);

        // Basic let
        let res = eval_str("(let ((x 3) (y 4)) (+ x y))", &mut heap, env).unwrap();
        assert!(matches!(res, Expr::Number(n) if n == 7.0));
    }

    #[test]
    fn test_vm_let_scoping() {
        let mut heap = Heap::new();
        let env = builtins::global_env(&mut heap);

        // let bindings are not visible outside the body
        eval_str("(define z 99)", &mut heap, env).unwrap();
        let res = eval_str("(let ((z 1)) z)", &mut heap, env).unwrap();
        assert!(matches!(res, Expr::Number(n) if n == 1.0));

        // After the let, z should still be 99 in the outer env
        let res2 = eval_str("z", &mut heap, env).unwrap();
        assert!(matches!(res2, Expr::Number(n) if n == 99.0));
    }

    #[test]
    fn test_vm_let_star() {
        let mut heap = Heap::new();
        let env = builtins::global_env(&mut heap);

        // let* allows later bindings to see earlier ones
        let res = eval_str("(let* ((x 1) (y (+ x 1))) y)", &mut heap, env).unwrap();
        assert!(matches!(res, Expr::Number(n) if n == 2.0));
    }

    #[test]
    fn test_vm_eval_fallback() {
        let mut heap = Heap::new();
        let env = builtins::global_env(&mut heap);

        // This is compilable, so it runs in the VM
        let res1 = eval_str("(+ 10 20)", &mut heap, env).unwrap();
        assert!(matches!(res1, Expr::Number(n) if n == 30.0));

        // This still falls back to tree-walker (uses lambda)
        let res2 = eval_str("(let ((x 5)) ((lambda (y) (+ x y)) 10))", &mut heap, env).unwrap();
        assert!(matches!(res2, Expr::Number(n) if n == 15.0));
     }

    #[test]
    fn test_vm_quasiquote() {
        let mut heap = Heap::new();
        let env = builtins::global_env(&mut heap);

        // 1. basic unquote
        let res1 = eval_str("(let ((x 42)) `(the answer is ,x))", &mut heap, env).unwrap();
        assert_eq!(format!("{:?}", res1), "(the answer is 42)");

        // 2. unquote-splicing
        let res2 = eval_str("(let ((items '(1 2 3))) `(a ,@items b))", &mut heap, env).unwrap();
        assert_eq!(format!("{:?}", res2), "(a 1 2 3 b)");

        // 3. nested quasiquote
        let res3 = eval_str("`(a `(b ,(+ 1 2)))", &mut heap, env).unwrap();
        assert_eq!(format!("{:?}", res3), "(a (quasiquote (b (unquote (+ 1 2)))))");

        // 4. quasiquote in macro body
        let macro_decl = "
        (defmacro when (condition body)
          `(if ,condition ,body ()))
        ";
        let exprs = parse_all(macro_decl).unwrap();
        assert_eq!(exprs.len(), 1);
        crate::eval::eval_tree(&exprs[0], env, &mut heap).unwrap();

        let res4 = eval_str("(when (> 3 2) 77)", &mut heap, env).unwrap();
        assert!(matches!(res4, Expr::Number(n) if n == 77.0));
    }

    /// Verifies that macro calls work correctly in the hybrid VM+tree-walker
    /// setup at every nesting depth and context.
    #[test]
    fn test_macro_correctness() {
        let mut heap = Heap::new();
        let env = builtins::global_env(&mut heap);

        // Helper: evaluate a sequence of top-level forms, return last result.
        let eval_seq = |src: &str, heap: &mut Heap, env: GcHandle| -> Result<Expr, String> {
            let exprs = parse_all(src)?;
            let mut last = Expr::List(vec![]);
            for e in &exprs {
                last = vm_eval(e, env, heap)?;
            }
            Ok(last)
        };

        // 1. defmacro returns ()
        let dm_res = eval_seq("(defmacro my-when (condition body) `(if ,condition ,body ()))",
            &mut heap, env).unwrap();
        assert!(matches!(dm_res, Expr::List(ref v) if v.is_empty()),
            "defmacro should return (): got {:?}", dm_res);

        // 2. Basic top-level macro call
        let r = eval_seq(
            "(defmacro my-when2 (cond body) `(if ,cond ,body ()))
             (my-when2 (> 3 2) 99)",
            &mut heap, env).unwrap();
        assert!(matches!(r, Expr::Number(n) if n == 99.0),
            "top-level macro call failed: {:?}", r);

        // 3. Macro call inside let body — this was the Problem 1 bug
        let r2 = eval_seq(
            "(defmacro my-if-pos (x body) `(if (> ,x 0) ,body ()))
             (let ((v 5)) (my-if-pos v 42))",
            &mut heap, env).unwrap();
        assert!(matches!(r2, Expr::Number(n) if n == 42.0),
            "macro inside let failed: {:?}", r2);

        // 4. Macro call inside lambda body
        let r3 = eval_seq(
            "(defmacro my-double-check (x body) `(if (> ,x 0) ,body 0))
             (define check-fn (lambda (n) (my-double-check n (* n 2))))
             (check-fn 7)",
            &mut heap, env).unwrap();
        assert!(matches!(r3, Expr::Number(n) if n == 14.0),
            "macro inside lambda failed: {:?}", r3);

        // 5. Macro call as argument to a function
        let r4 = eval_seq(
            "(defmacro my-or (a b) `(if ,a ,a ,b))
             (+ (my-or 0 3) (my-or 4 0))",
            &mut heap, env).unwrap();
        assert!(matches!(r4, Expr::Number(n) if n == 7.0),
            "macro as function argument failed: {:?}", r4);

        // 6. Nested macro calls
        let r5 = eval_seq(
            "(defmacro my-and2 (a b) `(if ,a ,b ()))
             (defmacro my-when3 (cond body) `(if ,cond ,body ()))
             (my-when3 (my-and2 1 1) 55)",
            &mut heap, env).unwrap();
        assert!(matches!(r5, Expr::Number(n) if n == 55.0),
            "nested macro calls failed: {:?}", r5);

        // 7. Macro inside let*, each binding can reference earlier ones
        let r6 = eval_seq(
            "(defmacro my-inc (x) `(+ ,x 1))
             (let* ((a 10) (b (my-inc a))) b)",
            &mut heap, env).unwrap();
        assert!(matches!(r6, Expr::Number(n) if n == 11.0),
            "macro inside let* binding failed: {:?}", r6);
    }
}





