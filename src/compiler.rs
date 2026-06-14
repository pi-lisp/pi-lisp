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
                        // At compile time we don't have the LexEnv. It's added at runtime by eval.
                        // We use a dummy LexEnv here because compile runs before eval.
                        // Actually, our AST Expr::Lambda requires an Rc<LexEnv>.
                        // We shouldn't generate closures during compilation. Compilation just produces the structure.
                        // Wait, what if we use a different structure for Lambda?
                        // Or we can just insert `Rc::new(LexEnv::Empty)` here, and `eval` will replace it.
                        return Ok(Expr::List(vec![
                            Expr::Symbol("lambda".into()),
                            Expr::Number(params.len() as f64), // Stash arity in the compiled AST instead of param list
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
                        // let is (let ((name expr)...) body...)
                        if list.len() < 3 {
                            return Err("let: expected (let ((name expr)...) body...)".into());
                        }
                        let mut compiled_bindings = vec![];
                        let mut new_names = vec![];
                        if let Expr::List(bindings) = &list[1] {
                            for b in bindings {
                                if let Expr::List(pair) = b {
                                    if let Expr::Symbol(name) = &pair[0] {
                                        let val = compile(&pair[1], env)?;
                                        compiled_bindings.push(Expr::List(vec![
                                            Expr::Symbol(name.clone()), // We keep name for potential debug or we can discard
                                            val
                                        ]));
                                        new_names.push(name.clone());
                                    }
                                }
                            }
                        }
                        let old_len = env.len();
                        for n in new_names {
                            env.push(n);
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
                    "define" | "defmacro" => {
                        // Only compile the body, the name is not lexically bound
                        let mut res = vec![list[0].clone(), list[1].clone()];
                        for e in &list[2..] {
                            res.push(compile(e, env)?);
                        }
                        return Ok(Expr::List(res));
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
