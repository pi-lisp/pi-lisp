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

        // env mutation / binders: uncompilable
        let expr3 = parse_all("(let ((x 1)) x)").unwrap().remove(0);
        assert!(!is_compilable(&expr3));

        let expr4 = parse_all("(define x 1)").unwrap().remove(0);
        assert!(!is_compilable(&expr4));

        let expr5 = parse_all("(lambda (x) x)").unwrap().remove(0);
        assert!(!is_compilable(&expr5));

        // quasiquote with unquote: uncompilable
        let expr6 = parse_all("`(1 ,x)").unwrap().remove(0);
        assert!(!is_compilable(&expr6));

        // quasiquote without unquote: compilable
        let expr7 = parse_all("`(1 2 3)").unwrap().remove(0);
        assert!(is_compilable(&expr7));
    }

    #[test]
    fn test_vm_eval_fallback() {
        let mut heap = Heap::new();
        let env = builtins::global_env(&mut heap);

        // This is compilable, so it runs in the VM
        let res1 = eval_str("(+ 10 20)", &mut heap, env).unwrap();
        assert!(matches!(res1, Expr::Number(n) if n == 30.0));

        // This is uncompilable (uses let/lambda), so it falls back to tree-walker and succeeds
        let res2 = eval_str("(let ((x 5)) ((lambda (y) (+ x y)) 10))", &mut heap, env).unwrap();
        assert!(matches!(res2, Expr::Number(n) if n == 15.0));
    }
}


