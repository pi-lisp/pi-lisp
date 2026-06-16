use crate::env::{env_get, env_set, Env};
use crate::expr::{is_truthy, Expr, LexEnv, is_sentinel_symbol};
use crate::macros::{eval_quasiquote, expand_macro};
use crate::compiler::compile;
use crate::reader::parse_params;
use std::rc::Rc;

/// Evaluates an expression in the given environment.
pub fn eval(expr: &Expr, env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    match expr {
        Expr::Number(_) => Ok(expr.clone()),
        Expr::Symbol(s) => {
            if is_sentinel_symbol(s) {
                Ok(expr.clone())
            } else {
                env_get(env, s)
            }
        }
        Expr::Index(i) => lex_env.get(*i).ok_or_else(|| format!("unbound index {}", i)),
        Expr::Func(_) | Expr::Lambda(..) | Expr::Macro(..) | Expr::Path(..) | Expr::Pi(..) | Expr::Sigma(..) | Expr::GlueType(..) | Expr::Glue(..) => Ok(expr.clone()),
        Expr::List(list) => {
            if list.is_empty() {
                return Ok(Expr::List(vec![]));
            }

            if let Expr::Symbol(op) = &list[0] {
                match op.as_str() {
                    "quote" => return Ok(list[1].clone()),
                    "quasiquote" => return eval_quasiquote(&list[1], env, lex_env, 1),
                    "unquote" => return Err("unquote outside quasiquote".into()),

                    "if" => return eval_if(list, env, lex_env),
                    "define" => return eval_define(list, env, lex_env),
                    "lambda" => return eval_lambda(list, env, lex_env),
                    "defmacro" => return eval_defmacro(list, env, lex_env),
                    "begin" => return eval_begin(list, env, lex_env),
                    "let" => return eval_let(list, env, lex_env),
                    "letrec" => return eval_letrec(list, env, lex_env),
                    "funext" => return eval_funext(list, env, lex_env),

                    "path" => return eval_path(list, env, lex_env),
                    "papply" => return eval_papply(list, env, lex_env),

                    "pi" => return eval_pi(list, env, lex_env),
                    "piapply" => return eval_piapply(list, env, lex_env),

                    "sigma" => return eval_sigma(list, env, lex_env),
                    "sigmacod" => return eval_sigmacod(list, env, lex_env),

                    "glue-type" => return eval_glue_type(list, env, lex_env),
                    "glue"      => return eval_glue(list, env, lex_env),
                    "unglue"    => return eval_unglue(list, env, lex_env),
                    "__Path__" => {
                        if list.len() != 2 {
                            return Err("__Path__: expected 1 argument".into());
                        }
                        let dom = eval(&list[1], env, lex_env)?;
                        return Ok(Expr::List(vec![Expr::Symbol("__Path__".into()), dom]));
                    }
                    "__Glue__" => {
                        if list.len() != 2 {
                            return Err("__Glue__: expected 1 argument".into());
                        }
                        let base = eval(&list[1], env, lex_env)?;
                        return Ok(Expr::List(vec![Expr::Symbol("__Glue__".into()), base]));
                    }
                    _ => {
                        // If `op` names a macro, expand (with raw, unevaluated
                        // argument expressions) and evaluate the result.
                        if let Ok(Expr::Macro(params, body)) = env_get(env, op) {
                            // Macros operate on surface AST, not Core AST. Wait! 
                            // `expand_macro` currently takes raw unevaluated args.
                            // But `list[1..]` are Core AST (compiled). This is okay since they are S-expressions.
                            // Let's expand, then compile, then evaluate.
                            let expanded = expand_macro(&params, &body, &list[1..])?;
                            let mut dummy_names = Vec::new();
                            let compiled = compile(&expanded, &mut dummy_names)?;
                            return eval(&compiled, env, lex_env);
                        }
                    }
                }
            }

            // Normal function application: evaluate operator and operands.
            let func = eval(&list[0], env, lex_env)?;
            let args: Result<Vec<Expr>, String> =
                list[1..].iter().map(|e| eval(e, env, lex_env)).collect();
            apply(func, &args?, env)
        }
    }
}

/// (if cond then [else])
fn eval_if(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    let cond = eval(&list[1], env, lex_env)?;
    if is_truthy(&cond) {
        eval(&list[2], env, lex_env)
    } else if list.len() > 3 {
        eval(&list[3], env, lex_env)
    } else {
        Ok(Expr::List(vec![]))
    }
}

/// (define name expr)
fn eval_define(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    if let Expr::Symbol(name) = &list[1] {
        let val = eval(&list[2], env, lex_env)?;
        env_set(env, name.clone(), val.clone());
        Ok(val)
    } else {
        Err("invalid define: expected (define <symbol> <expr>)".into())
    }
}

/// (lambda arity body)
fn eval_lambda(list: &[Expr], _env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    if let Expr::Number(arity) = &list[1] {
        Ok(Expr::Lambda(*arity as usize, Box::new(list[2].clone()), lex_env.clone()))
    } else {
        Err("lambda core: expected arity".into())
    }
}

/// (path 1.0 body)
fn eval_path(list: &[Expr], _env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    Ok(Expr::Path(Box::new(list[2].clone()), lex_env.clone()))
}

/// (papply p t)
///
/// Applies a path `p` at interval coordinate `t`, where `t` must be a
/// number in [0,1]. `t = 0` and `t = 1` recover the path's endpoints;
/// interior values give whatever interpolation `body` computes.
fn eval_papply(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    if list.len() != 3 {
        return Err("papply: expected (papply <path> <interval-point>)".into());
    }
    let p = eval(&list[1], env, lex_env)?;
    let t = eval(&list[2], env, lex_env)?;

    let t_val = match &t {
        Expr::Number(n) => *n,
        other => return Err(format!("papply: interval point must be a number, got {:?}", other)),
    };
    if !(0.0..=1.0).contains(&t_val) {
        return Err(format!(
            "papply: interval point {} out of bounds, expected [0,1]",
            t_val
        ));
    }

    match p {
        Expr::Path(body, penv) => {
            let new_env = Rc::new(LexEnv::Node(Expr::Number(t_val), penv));
            eval(&body, env, &new_env)
        }
        other => Err(format!("papply: not a path: {:?}", other)),
    }
}

/// (pi (x) dom cod)
///
/// Introduces a dependent function type (Π-type): the type of functions
/// from `dom` to `cod(x)`, where `cod` may mention the bound variable `x`.
///
/// Usage examples:
///   `(pi (x) Nat Nat)`         -- the non-dependent arrow Nat → Nat
///   `(pi (x) Nat (Vec x))`     -- the type of vectors of length x
///   `(piapply (pi (x) Nat Nat) 3)` -- instantiates the codomain at 3, => Nat
fn eval_pi(list: &[Expr], _env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    let dom = Box::new(list[1].clone());
    let cod = Box::new(list[2].clone());
    Ok(Expr::Pi(dom, cod, lex_env.clone()))
}

/// (piapply p v)
///
/// Instantiates a Pi-type `p` at value `v`, evaluating the codomain
/// expression with the bound variable set to `v`.  This gives the
/// *type* of `p`-typed functions applied to a concrete argument value.
///
/// For a non-dependent arrow `(pi (x) A B)`, `piapply` always returns
/// (the evaluation of) `B` regardless of `v`.  For genuinely dependent
/// types, the returned type will vary with `v`.
fn eval_piapply(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    if list.len() != 3 {
        return Err("piapply: expected (piapply <pi-type> <value>)".into());
    }
    let p = eval(&list[1], env, lex_env)?;
    let v = eval(&list[2], env, lex_env)?;

    match p {
        Expr::Pi(_dom, cod, penv) => {
            let new_env = Rc::new(LexEnv::Node(v, penv));
            eval(&cod, env, &new_env)
        }
        other => Err(format!("piapply: not a pi-type: {:?}", other)),
    }
}

/// (sigma (x) dom cod)
///
/// Introduces a dependent pair type (Σ-type): the type of pairs where
/// the first component has type `dom` and the second component has type `cod(x)`,
/// where `cod` may mention the bound variable `x` (the first component).
fn eval_sigma(list: &[Expr], _env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    let dom = Box::new(list[1].clone());
    let cod = Box::new(list[2].clone());
    Ok(Expr::Sigma(dom, cod, lex_env.clone()))
}

/// (sigmacod s v)
///
/// Instantiates a Sigma-type `s` at value `v` (which should be the first
/// component of a pair), evaluating the codomain expression with the bound
/// variable set to `v`. This gives the *type* of the second component.
fn eval_sigmacod(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    if list.len() != 3 {
        return Err("sigmacod: expected (sigmacod <sigma-type> <value>)".into());
    }
    let s = eval(&list[1], env, lex_env)?;
    let v = eval(&list[2], env, lex_env)?;

    match s {
        Expr::Sigma(_dom, cod, penv) => {
            let new_env = Rc::new(LexEnv::Node(v, penv));
            eval(&cod, env, &new_env)
        }
        other => Err(format!("sigmacod: not a sigma-type: {:?}", other)),
    }
}

/// (defmacro name (params...) body)
fn eval_defmacro(list: &[Expr], env: &Env, _lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    if let Expr::Symbol(name) = &list[1] {
        let params = parse_params(&list[2])?;
        let mac = Expr::Macro(params, Box::new(list[3].clone()));
        env_set(env, name.clone(), mac.clone());
        Ok(mac)
    } else {
        Err("invalid defmacro: expected <symbol>".into())
    }
}

/// (begin expr...)
fn eval_begin(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    let mut result = Expr::List(vec![]);
    for e in &list[1..] {
        result = eval(e, env, lex_env)?;
    }
    Ok(result)
}

/// (let ((name expr)...) body...)
fn eval_let(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    // Bindings are sequential: each RHS is evaluated in the environment
    // extended by all *preceding* bindings (left-to-right), matching the
    // compiler's De Bruijn index assignment order.
    let mut current_env = lex_env.clone();
    if let Expr::List(bindings) = &list[1] {
        for b in bindings {
            if let Expr::List(pair) = b {
                // Evaluate RHS in the env so far (not the outer lex_env),
                // so that De Bruijn indices referring to earlier bindings work.
                let val = eval(&pair[1], env, &current_env)?;
                current_env = Rc::new(LexEnv::Node(val, current_env));
            }
        }
    }
    let mut result = Expr::List(vec![]);
    for e in &list[2..] {
        result = eval(e, env, &current_env)?;
    }
    Ok(result)
}

/// (letrec ((name expr)...) body...)
///
/// All binding names are in scope for every RHS and the body, enabling mutual
/// recursion. We implement this with the standard "back-patch" trick:
///   1. Pre-populate the lex env with placeholder values for every binding.
///   2. Evaluate each RHS in that extended env (forward references hit the
///      placeholder, which is fine for lambdas since the body isn't called yet).
///   3. Evaluate the body in an env where the placeholders are replaced by the
///      real values.
///
/// Because `LexEnv` is an immutable `Rc` chain we can't truly back-patch in
/// place. Instead we build a two-pass scheme: first extend with placeholders,
/// then rebuild the chain with the real values. Self-referential lambdas
/// close over the placeholder env on pass 1, so recursive calls during
/// execution will see the placeholder — to fix that we re-evaluate lambdas
/// in the fully-resolved env so they capture the right closure.
fn eval_letrec(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    if list.len() < 3 {
        return Err("letrec: expected bindings and a body".into());
    }
    let bindings = if let Expr::List(b) = &list[1] { b } else {
        return Err("letrec: bindings must be a list".into());
    };
    let n = bindings.len();

    // Pass 1: extend lex_env with `n` placeholder symbols so that any
    // lambda body compiled for these bindings can reference all names via
    // their De Bruijn index without an "unbound index" error.
    let placeholder = Expr::Symbol("__letrec_placeholder__".into());
    let mut placeholder_env = lex_env.clone();
    for _ in 0..n {
        placeholder_env = Rc::new(LexEnv::Node(placeholder.clone(), placeholder_env));
    }

    // Pass 2: evaluate every RHS in the placeholder env.
    let mut vals: Vec<Expr> = Vec::with_capacity(n);
    for b in bindings {
        if let Expr::List(pair) = b {
            if pair.len() >= 2 {
                vals.push(eval(&pair[1], env, &placeholder_env)?);
            } else {
                return Err("letrec: each binding must be (name expr)".into());
            }
        } else {
            return Err("letrec: each binding must be a list".into());
        }
    }

    // Pass 3: build the real env with the concrete values.
    // The compiler pushed names left-to-right, so the *first* binding ends up
    // at the deepest index and the *last* at index 0. We push in the same order
    // so that De Bruijn indices computed by the compiler are correct.
    let mut real_env = lex_env.clone();
    for v in &vals {
        real_env = Rc::new(LexEnv::Node(v.clone(), real_env));
    }

    // Re-close any lambdas captured in the placeholder env so recursive calls
    // see the real bindings instead of the placeholder.
    let reclose = |v: Expr| -> Expr {
        match v {
            Expr::Lambda(arity, body, _) => Expr::Lambda(arity, body, real_env.clone()),
            other => other,
        }
    };
    // Rebuild real_env with re-closed lambdas.
    let mut final_env = lex_env.clone();
    for v in vals {
        final_env = Rc::new(LexEnv::Node(reclose(v), final_env));
    }

    let mut result = Expr::List(vec![]);
    for e in &list[2..] {
        result = eval(e, env, &final_env)?;
    }
    Ok(result)
}

/// (funext f g p)
///
/// Function extensionality: given
///   f g : Π(x : A), B x
///   p   : Π(x : A), Path(B x) (f x) (g x)   -- a pointwise homotopy
///
/// produces a `Path` value between `f` and `g` such that applying the path
/// at any interval point `i` returns a function `λ x. (p x) @ i`.
///
/// The resulting path is represented as:
///   Path(λ i. λ x. (papply (p x) i))
fn eval_funext(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    if list.len() != 4 {
        return Err("funext: expected (funext f g p)".into());
    }
    // We don't need f or g at runtime — the path is fully determined by p.
    let p = eval(&list[3], env, lex_env)?;

    // Build a Path whose body, when applied at interval `i` (De Bruijn #0),
    // is `λ x. papply (p x) i`.
    //
    // In the path body the interval variable `i` is at De Bruijn index 0
    // (pushed by papply/path), and in the inner lambda `x` is at index 0
    // (with `i` shifted to index 1). We avoid representing this as a
    // compiled Core expression and instead build a runtime closure directly.
    let p_clone = p.clone();
    let env_clone = env.clone();
    // The path body is a Func that, given the interval value i,
    // returns a lambda over x such that applying p to x and then the path at i.
    let body_fn = Expr::Func(Rc::new(move |args: &[Expr]| -> Result<Expr, String> {
        let i = args[0].clone();
        let p_inner = p_clone.clone();
        let env_inner = env_clone.clone();
        // Return λ(x). papply (p x) i
        let i_inner = i.clone();
        Ok(Expr::Func(Rc::new(move |xargs: &[Expr]| -> Result<Expr, String> {
            let x = xargs[0].clone();
            // Evaluate (p x)
            let px = apply(p_inner.clone(), &[x], &env_inner)?;
            // Evaluate (papply px i)
            match px {
                Expr::Path(body, penv) => {
                    let new_lex = Rc::new(LexEnv::Node(i_inner.clone(), penv));
                    eval(&body, &env_inner, &new_lex)
                }
                other => Err(format!("funext: pointwise homotopy did not return a path, got {:?}", other)),
            }
        })))
    }));

    // Wrap in a Path that holds this body function and the current lex_env.
    // We represent the body as a synthetic lambda arity-1 over the interval,
    // using a Func so we don't need to build Core AST.
    Ok(Expr::Path(Box::new(body_fn), lex_env.clone()))
}

/// (glue-type base equiv)
///
/// Constructs a Glue type: `base` is the underlying type A, and `equiv` is a
/// function f : B → A witnessing that B is (fibered) equivalent to A along
/// some face. The resulting `GlueType` value acts as a type whose terms are
/// `Glue` introductions that pair a B-value with the equivalence.
fn eval_glue_type(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    if list.len() != 3 {
        return Err("glue-type: expected (glue-type <base-type> <equiv>)".into());
    }
    let base  = eval(&list[1], env, lex_env)?;
    let equiv = eval(&list[2], env, lex_env)?;
    Ok(Expr::GlueType(Box::new(base), Box::new(equiv)))
}

/// (glue val equiv)
///
/// Introduction form for Glue types. `val` is the B-side fiber value and
/// `equiv` is the forward function f : B → A. The result is a `Glue` term
/// that remembers both so that `unglue` can recover the A-side image.
fn eval_glue(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    if list.len() != 3 {
        return Err("glue: expected (glue <val> <equiv>)".into());
    }
    let val   = eval(&list[1], env, lex_env)?;
    let equiv = eval(&list[2], env, lex_env)?;
    Ok(Expr::Glue(Box::new(val), Box::new(equiv)))
}

/// (unglue g)
///
/// Elimination form for Glue terms. Applies the stored equivalence function
/// to the stored fiber value, projecting back into the base type A.
/// That is: `(unglue (glue v f))` reduces to `(f v)`.
fn eval_unglue(list: &[Expr], env: &Env, lex_env: &Rc<LexEnv>) -> Result<Expr, String> {
    if list.len() != 2 {
        return Err("unglue: expected (unglue <glue-term>)".into());
    }
    let g = eval(&list[1], env, lex_env)?;
    match g {
        Expr::Glue(val, equiv) => apply(*equiv, &[*val], env),
        other => Err(format!("unglue: not a glue term: {:?}", other)),
    }
}

/// Applies a function (builtin or lambda) to already-evaluated arguments.
pub fn apply(func: Expr, args: &[Expr], env: &Env) -> Result<Expr, String> {
    match func {
        Expr::Func(f) => f(args),
        Expr::Lambda(arity, body, penv) => {
            if args.len() != arity {
                return Err(format!("lambda expected {} args, got {}", arity, args.len()));
            }
            // Bind arguments from left to right.
            // The compiler pushed params from left to right. Thus, the last parameter
            // corresponds to the highest index (i.e. pushed last).
            let mut current_env = penv;
            for arg in args {
                current_env = Rc::new(LexEnv::Node(arg.clone(), current_env));
            }
            eval(&body, env, &current_env)
        }
        other => Err(format!("not a function: {:?}", other)),
    }
}