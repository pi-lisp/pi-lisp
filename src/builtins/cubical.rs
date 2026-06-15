use std::rc::Rc;

use crate::{env::{Env, env_set}, expr::Expr};

pub fn register_intervals(env: &Env) {
    // The two canonical endpoints of the interval I = [0,1].
    env_set(env, "i0".into(), Expr::Number(0.0));
    env_set(env, "i1".into(), Expr::Number(1.0));

    // (refl x): the constant path at x, i.e. a path that ignores its
    // interval argument and always evaluates to x. This is the cubical
    // "reflexivity" path -- evidence that x equals itself, viewed as a
    // degenerate line I -> A.
    env_set(
        env,
        "refl".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("refl: expects exactly 1 argument".into());
            }
            // The body is `(quote x)` so that re-evaluating it always
            // yields the value `x` unchanged.
            Ok(Expr::Path(
                Box::new(Expr::List(vec![
                    Expr::Symbol("quote".into()),
                    args[0].clone(),
                ])),
                Rc::new(crate::expr::LexEnv::Empty),
            ))
        })),
    );
}

pub fn register_pi_types(env: &Env) {
    // (pi? x) -- returns 1 if x is a Pi-type value, 0 otherwise.
    // Useful for runtime type inspection / dispatch.
    env_set(
        env,
        "pi?".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("pi?: expects exactly 1 argument".into());
            }
            Ok(Expr::Number(match &args[0] {
                Expr::Pi(..) => 1.0,
                _ => 0.0,
            }))
        })),
    );

    // (path? x) -- returns 1 if x is a Path value, 0 otherwise.
    env_set(
        env,
        "path?".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("path?: expects exactly 1 argument".into());
            }
            Ok(Expr::Number(match &args[0] {
                Expr::Path(..) => 1.0,
                _ => 0.0,
            }))
        })),
    );
}

pub fn register_sigma_types(env: &Env) {
    // (sigma? x) -- returns 1 if x is a Sigma-type value, 0 otherwise.
    env_set(
        env,
        "sigma?".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("sigma?: expects exactly 1 argument".into());
            }
            Ok(Expr::Number(match &args[0] {
                Expr::Sigma(..) => 1.0,
                _ => 0.0,
            }))
        })),
    );
}

pub fn register_glue_types(env: &Env) {
    // (glue? x) -- returns 1 if x is a Glue introduction term, 0 otherwise.
    env_set(
        env,
        "glue?".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("glue?: expects exactly 1 argument".into());
            }
            Ok(Expr::Number(match &args[0] {
                Expr::Glue(..) => 1.0,
                _ => 0.0,
            }))
        })),
    );

    // (glue-type? x) -- returns 1 if x is a GlueType type former, 0 otherwise.
    env_set(
        env,
        "glue-type?".into(),
        Expr::Func(Rc::new(|args| {
            if args.len() != 1 {
                return Err("glue-type?: expects exactly 1 argument".into());
            }
            Ok(Expr::Number(match &args[0] {
                Expr::GlueType(..) => 1.0,
                _ => 0.0,
            }))
        })),
    );
}