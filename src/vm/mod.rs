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
    // ── Macro-call fast-path ─────────────────────────────────────────────────
    // If the expression is a list whose head is a symbol that resolves to a
    // Macro in the current environment, hand it off to the tree-walker.  The
    // tree-walker correctly expands the macro and re-evaluates the expanded
    // form, whereas the VM compiler would evaluate the quasiquote template at
    // compile time and return the expanded list as data.
    if let Expr::List(items) = expr {
        if let Some(Expr::Symbol(head)) = items.first() {
            use crate::expr::env_get;
            if let Ok(Expr::Macro(..)) = env_get(heap, env, head) {
                return tree_eval(expr, env, heap);
            }
        }
    }

    if !is_compilable(expr) {
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

        // Simple arithmetic/comparisons: compilable
        let expr1 = parse_all("(+ 1 2)").unwrap().remove(0);
        assert!(is_compilable(&expr1));

        let expr2 = parse_all("(if (= 1 1) (+ 2 3) 4)").unwrap().remove(0);
        assert!(is_compilable(&expr2));

        // define, let, let* are now compilable
        let expr3 = parse_all("(let ((x 1)) x)").unwrap().remove(0);
        assert!(is_compilable(&expr3), "let should now be compilable");

        let expr4 = parse_all("(define x 1)").unwrap().remove(0);
        assert!(is_compilable(&expr4), "define should now be compilable");

        let expr5 = parse_all("(let* ((x 1) (y (+ x 1))) y)").unwrap().remove(0);
        assert!(is_compilable(&expr5), "let* should now be compilable");

        // lambda is still uncompilable
        let expr_lambda = parse_all("(lambda (x) x)").unwrap().remove(0);
        assert!(is_compilable(&expr_lambda));

        // quasiquote with unquote: uncompilable
        let expr6 = parse_all("`(1 ,x)").unwrap().remove(0);
        assert!(!is_compilable(&expr6));

        // quasiquote without unquote: compilable
        let expr7 = parse_all("`(1 2 3)").unwrap().remove(0);
        assert!(is_compilable(&expr7));
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
}


