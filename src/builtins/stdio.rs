use std::rc::Rc;

use crate::{env::{Env, env_set}, expr::Expr};

pub fn register_misc(env: &Env) {
    env_set(
        env,
        "print".into(),
        Expr::Func(Rc::new(|args| {
            for a in args {
                print!("{:?} ", a);
            }
            println!();
            Ok(Expr::List(vec![]))
        })),
    );

    env_set(
        env,
        "read".into(),
        Expr::Func(Rc::new(|args| {
            if !args.is_empty() {
                return Err("read expects 0 arguments".into());
            }
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
            let exprs = crate::reader::parse_all(&input).map_err(|e| format!("read parse error: {}", e))?;
            if exprs.is_empty() {
                Ok(Expr::List(vec![]))
            } else {
                Ok(exprs[0].clone())
            }
        })),
    );

    env_set(
        env,
        "write".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("write expects exactly 1 argument".into());
            }
            print!("{:?}", args[0]);
            use std::io::Write;
            std::io::stdout().flush().map_err(|e| e.to_string())?;
            Ok(Expr::List(vec![]))
        })),
    );

    env_set(
        env,
        "newline".into(),
        Expr::Func(Rc::new(|args| {
            if !args.is_empty() {
                return Err("newline expects 0 arguments".into());
            }
            println!();
            Ok(Expr::List(vec![]))
        })),
    );
}