use crate::expr::Expr;
use crate::reader::parse_params;

/// Compiles a "surface" AST (with named variables) into a "core" AST (with De Bruijn indices).
pub fn compile(expr: &Expr, env: &mut Vec<String>) -> Result<Expr, String> {
    match expr {
        Expr::Number(_) => Ok(expr.clone()),
        Expr::Symbol(s) => {
            // Find the index of the variable from the back of the lexical environment.
            if let Some(pos) = env.iter().rev().position(|name| name == s) {
                Ok(Expr::Index(pos))
            } else {
                // If it's not in the lexical environment, it's a global variable or built-in,
                // so leave it as a Symbol.
                Ok(expr.clone())
            }
        }
        Expr::List(list) => {
            if list.is_empty() {
                return Ok(expr.clone());
            }
            if let Expr::Symbol(op) = &list[0] {
                match op.as_str() {
                    "quote" => return Ok(expr.clone()), // Quote is untouched
                    "quasiquote" => {
                        // For quasiquote, we must carefully compile unquoted expressions
                        // but not the quoted parts. Let's do a simple recursive descent.
                        return compile_quasiquote(expr, env, 1);
                    }
                    "lambda" => {
                        if list.len() < 3 {
                            return Err("lambda: expected (lambda (params...) body)".into());
                        }
                        let params = parse_params(&list[1])?;
                        let old_len = env.len();
                        for p in &params {
                            env.push(p.clone());
                        }
                        let body = compile(&list[2], env)?;
                        env.truncate(old_len);
                        return Ok(Expr::List(vec![
                            Expr::Symbol("lambda".into()),
                            Expr::Number(params.len() as f64),
                            body,
                        ]));
                    }
                    "path" => {
                        let params = parse_params(&list[1])?;
                        if params.len() != 1 {
                            return Err("path: expected exactly one interval variable".into());
                        }
                        let old_len = env.len();
                        env.push(params[0].clone());
                        let body = compile(&list[2], env)?;
                        env.truncate(old_len);
                        return Ok(Expr::List(vec![
                            Expr::Symbol("path".into()),
                            Expr::Number(1.0),
                            body,
                        ]));
                    }
                    "pi" => {
                        let params = parse_params(&list[1])?;
                        if params.len() != 1 {
                            return Err("pi: expected exactly one bound variable".into());
                        }
                        let dom = compile(&list[2], env)?;
                        let old_len = env.len();
                        env.push(params[0].clone());
                        let cod = compile(&list[3], env)?;
                        env.truncate(old_len);
                        return Ok(Expr::List(vec![
                            Expr::Symbol("pi".into()),
                            dom,
                            cod,
                        ]));
                    }
                    "sigma" => {
                        let params = parse_params(&list[1])?;
                        if params.len() != 1 {
                            return Err("sigma: expected exactly one bound variable".into());
                        }
                        let dom = compile(&list[2], env)?;
                        let old_len = env.len();
                        env.push(params[0].clone());
                        let cod = compile(&list[3], env)?;
                        env.truncate(old_len);
                        return Ok(Expr::List(vec![
                            Expr::Symbol("sigma".into()),
                            dom,
                            cod,
                        ]));
                    }
                    "let" => {
                        // (let ((name expr)...) body...)
                        // Each binding's RHS is compiled in the environment *before* that
                        // binding is added (sequential, non-recursive). This preserves the
                        // correct De Bruijn indices: a later binding can shadow an earlier
                        // one, but not refer to it by the same name in its own RHS.
                        if list.len() < 3 {
                            return Err("let: expected (let ((name expr)...) body...)".into());
                        }
                        let mut compiled_bindings = vec![];
                        let old_len = env.len();
                        if let Expr::List(bindings) = &list[1] {
                            for b in bindings {
                                if let Expr::List(pair) = b {
                                    if pair.len() != 2 {
                                        return Err("let: each binding must be (name expr)".into());
                                    }
                                    if let Expr::Symbol(name) = &pair[0] {
                                        // RHS compiled BEFORE pushing name, so it cannot
                                        // accidentally refer to itself (use letrec for that).
                                        let val = compile(&pair[1], env)?;
                                        compiled_bindings.push(Expr::List(vec![
                                            Expr::Symbol(name.clone()),
                                            val,
                                        ]));
                                        env.push(name.clone());
                                    } else {
                                        return Err("let: binding name must be a symbol".into());
                                    }
                                } else {
                                    return Err("let: each binding must be a list (name expr)".into());
                                }
                            }
                        }
                        let mut compiled_body = vec![];
                        for e in &list[2..] {
                            compiled_body.push(compile(e, env)?);
                        }
                        env.truncate(old_len);

                        let mut res = vec![Expr::Symbol("let".into()), Expr::List(compiled_bindings)];
                        res.extend(compiled_body);
                        return Ok(Expr::List(res));
                    }
                    "letrec" => {
                        // (letrec ((name expr)...) body...)
                        // All names are in scope for ALL RHSes and the body, enabling mutual
                        // recursion and self-referential bindings (e.g. recursive functions).
                        // The evaluator must handle the forward references at runtime.
                        if list.len() < 3 {
                            return Err("letrec: expected (letrec ((name expr)...) body...)".into());
                        }
                        let old_len = env.len();
                        let mut names = vec![];
                        if let Expr::List(bindings) = &list[1] {
                            for b in bindings {
                                if let Expr::List(pair) = b {
                                    if let Expr::Symbol(name) = &pair[0] {
                                        names.push(name.clone());
                                    } else {
                                        return Err("letrec: binding name must be a symbol".into());
                                    }
                                }
                            }
                        }
                        // Push ALL names first so every RHS can see every name.
                        for n in &names {
                            env.push(n.clone());
                        }
                        let mut compiled_bindings = vec![];
                        if let Expr::List(bindings) = &list[1] {
                            for b in bindings {
                                if let Expr::List(pair) = b {
                                    if let Expr::Symbol(name) = &pair[0] {
                                        let val = compile(&pair[1], env)?;
                                        compiled_bindings.push(Expr::List(vec![
                                            Expr::Symbol(name.clone()),
                                            val,
                                        ]));
                                    }
                                }
                            }
                        }
                        let mut compiled_body = vec![];
                        for e in &list[2..] {
                            compiled_body.push(compile(e, env)?);
                        }
                        env.truncate(old_len);

                        let mut res = vec![Expr::Symbol("letrec".into()), Expr::List(compiled_bindings)];
                        res.extend(compiled_body);
                        return Ok(Expr::List(res));
                    }
                    "define" | "defmacro" => {
                        // Only compile the body, the name is not lexically bound
                        let mut res = vec![list[0].clone(), list[1].clone()];
                        for e in &list[2..] {
                            res.push(compile(e, env)?);
                        }
                        return Ok(Expr::List(res));
                    }
                    // ---------------------------------------------------------------
                    // Function extensionality
                    //
                    // (funext (x) f g p) compiles to a path between f and g given
                    // a pointwise path p : Π(x : A), Path(f x, g x).
                    //
                    //   f g : Π(x : A), B x
                    //   p   : (lambda (x) (path (i) ...))     -- pointwise homotopy
                    //
                    // The compiled form is (funext <compiled-f> <compiled-g> <compiled-p>)
                    // and the evaluator is expected to construct a Path value whose
                    // application at each endpoint reduces to f / g.
                    // ---------------------------------------------------------------
                    "funext" => {
                        if list.len() != 4 {
                            return Err(
                                "funext: expected (funext f g p) where p : Π x, Path (f x) (g x)"
                                    .into(),
                            );
                        }
                        let f = compile(&list[1], env)?;
                        let g = compile(&list[2], env)?;
                        let p = compile(&list[3], env)?;
                        return Ok(Expr::List(vec![
                            Expr::Symbol("funext".into()),
                            f,
                            g,
                            p,
                        ]));
                    }
                    _ => {}
                }
            }

            // Default: compile every element in the list
            let mut result = Vec::new();
            for item in list {
                result.push(compile(item, env)?);
            }
            Ok(Expr::List(result))
        }
        _ => Ok(expr.clone()),
    }
}

fn compile_quasiquote(expr: &Expr, env: &mut Vec<String>, depth: i32) -> Result<Expr, String> {
    match expr {
        Expr::List(list) if !list.is_empty() => {
            if let Expr::Symbol(s) = &list[0] {
                if s == "unquote" {
                    if depth == 1 {
                        return Ok(Expr::List(vec![
                            Expr::Symbol("unquote".into()),
                            compile(&list[1], env)?,
                        ]));
                    } else {
                        return Ok(Expr::List(vec![
                            Expr::Symbol("unquote".into()),
                            compile_quasiquote(&list[1], env, depth - 1)?,
                        ]));
                    }
                }
                if s == "quasiquote" {
                    return Ok(Expr::List(vec![
                        Expr::Symbol("quasiquote".into()),
                        compile_quasiquote(&list[1], env, depth + 1)?,
                    ]));
                }
            }

            let mut result = Vec::new();
            for item in list {
                if let Expr::List(inner) = item {
                    if inner.len() == 2 {
                        if let Expr::Symbol(s) = &inner[0] {
                            if s == "unquote-splicing" {
                                if depth == 1 {
                                    result.push(Expr::List(vec![
                                        Expr::Symbol("unquote-splicing".into()),
                                        compile(&inner[1], env)?,
                                    ]));
                                    continue;
                                }
                            }
                        }
                    }
                }
                result.push(compile_quasiquote(item, env, depth)?);
            }
            Ok(Expr::List(result))
        }
        _ => Ok(expr.clone()),
    }
}