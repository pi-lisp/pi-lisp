use crate::expr::Expr;

/// Splits source text into tokens. Handles parens and the `'` quote shorthand.
pub fn tokenize(src: &str) -> Vec<String> {
    src.replace('(', " ( ")
        .replace(')', " ) ")
        .replace('\'', " ' ")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Parses a single expression starting at `*pos`, advancing `*pos` past it.
pub fn parse(tokens: &[String], pos: &mut usize) -> Result<Expr, String> {
    let tok = tokens.get(*pos).ok_or("unexpected EOF")?;
    *pos += 1;
    match tok.as_str() {
        "(" => {
            let mut list = Vec::new();
            loop {
                if tokens.get(*pos).map(|s| s.as_str()) == Some(")") {
                    *pos += 1;
                    break;
                }
                if *pos >= tokens.len() {
                    return Err("unexpected EOF in list".into());
                }
                list.push(parse(tokens, pos)?);
            }
            Ok(Expr::List(list))
        }
        ")" => Err("unexpected )".into()),
        "'" => {
            // 'expr  =>  (quote expr)
            let inner = parse(tokens, pos)?;
            Ok(Expr::List(vec![Expr::Symbol("quote".into()), inner]))
        }
        _ => {
            if let Ok(n) = tok.parse::<f64>() {
                Ok(Expr::Number(n))
            } else {
                Ok(Expr::Symbol(tok.clone()))
            }
        }
    }
}

/// Parses an entire source string into a sequence of top-level expressions.
pub fn parse_all(src: &str) -> Result<Vec<Expr>, String> {
    let tokens = tokenize(src);
    let mut pos = 0;
    let mut exprs = Vec::new();
    while pos < tokens.len() {
        exprs.push(parse(&tokens, &mut pos)?);
    }
    Ok(exprs)
}

/// Convenience helper: parses params list `(a b c)` into Vec<String>.
/// Returns an error if any element is not a symbol, so callers get a clear
/// diagnostic instead of a silently-empty string that corrupts arity counts
/// for curried lambdas compiled with De Bruijn indices.
pub fn parse_params(e: &Expr) -> Result<Vec<String>, String> {
    if let Expr::List(p) = e {
        p.iter()
            .map(|e| match e {
                Expr::Symbol(s) => Ok(s.clone()),
                other => Err(format!(
                    "parse_params: expected symbol in parameter list, got {:?}",
                    other
                )),
            })
            .collect()
    } else if let Expr::Symbol(s) = e {
        // Allow a bare symbol as a single-param shorthand: (lambda x body)
        Ok(vec![s.clone()])
    } else {
        Ok(vec![])
    }
}