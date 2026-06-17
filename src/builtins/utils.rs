use std::rc::Rc;

use crate::{builtins::{display_str, num, str_arg}, env::{Env, env_set}, expr::Expr};

pub fn register_strings(env: &Env) {
    env_set(
        env,
        "string?".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("string?: expects exactly 1 argument".into());
            }
            Ok(Expr::Number(if let Expr::Str(_) = &args[0] {
                1.0
            } else {
                0.0
            }))
        })),
    );

    env_set(
        env,
        "string-append".into(),
        Expr::Func(Rc::new(|args| {
            let mut out = String::new();
            for a in args {
                out.push_str(str_arg(a)?);
            }
            Ok(Expr::Str(out))
        })),
    );

    env_set(
        env,
        "string-length".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("string-length: expects exactly 1 argument".into());
            }
            Ok(Expr::Number(str_arg(&args[0])?.chars().count() as f64))
        })),
    );

    macro_rules! string_cmp_fn {
        ($op:tt) => {
            Expr::Func(Rc::new(|args| {
                if args.len() != 2 {
                    return Err("string comparison expects exactly 2 arguments".into());
                }
                let a = str_arg(&args[0])?;
                let b = str_arg(&args[1])?;
                Ok(Expr::Number(if a $op b { 1.0 } else { 0.0 }))
            }))
        };
    }

    env_set(env, "string=?".into(), string_cmp_fn!(==));
    env_set(env, "string<?".into(), string_cmp_fn!(<));
    env_set(env, "string>?".into(), string_cmp_fn!(>));
    env_set(env, "string<=?".into(), string_cmp_fn!(<=));
    env_set(env, "string>=?".into(), string_cmp_fn!(>=));

    env_set(
        env,
        "string->number".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("string->number: expects exactly 1 argument".into());
            }
            let s = str_arg(&args[0])?;
            s.parse::<f64>()
                .map(Expr::Number)
                .map_err(|_| format!("string->number: not a valid number: {:?}", s))
        })),
    );

    env_set(
        env,
        "number->string".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("number->string: expects exactly 1 argument".into());
            }
            Ok(Expr::Str(format!("{}", num(&args[0])?)))
        })),
    );

    env_set(
        env,
        "string->symbol".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("string->symbol: expects exactly 1 argument".into());
            }
            Ok(Expr::Symbol(str_arg(&args[0])?.to_string()))
        })),
    );

    env_set(
        env,
        "symbol->string".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("symbol->string: expects exactly 1 argument".into());
            }
            match &args[0] {
                Expr::Symbol(s) => Ok(Expr::Str(s.clone())),
                other => Err(format!("symbol->string: expected symbol, got {:?}", other)),
            }
        })),
    );

    env_set(
        env,
        "string-upcase".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("string-upcase: expects exactly 1 argument".into());
            }
            Ok(Expr::Str(str_arg(&args[0])?.to_uppercase()))
        })),
    );

    env_set(
        env,
        "string-downcase".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("string-downcase: expects exactly 1 argument".into());
            }
            Ok(Expr::Str(str_arg(&args[0])?.to_lowercase()))
        })),
    );

    // (substring s start end) — character-indexed, end-exclusive, like Scheme.
    env_set(
        env,
        "substring".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 3 {
                return Err("substring: expects (substring s start end)".into());
            }
            let s = str_arg(&args[0])?;
            let start = num(&args[1])? as usize;
            let end = num(&args[2])? as usize;
            let chars: Vec<char> = s.chars().collect();
            if start > end || end > chars.len() {
                return Err(format!(
                    "substring: index out of range (start={}, end={}, len={})",
                    start,
                    end,
                    chars.len()
                ));
            }
            Ok(Expr::Str(chars[start..end].iter().collect()))
        })),
    );
}

pub fn register_misc(env: &Env) {
    env_set(
        env,
        "print".into(),
        Expr::Func(Rc::new(|args| {
            for a in args {
                print!("{} ", display_str(a));
            }
            println!();
            Ok(Expr::List(vec![]))
        })),
    );
}

