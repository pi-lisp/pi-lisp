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
pub fn parse_params(e: &Expr) -> Result<Vec<String>, String> {
    if let Expr::List(p) = e {
        Ok(p
            .iter()
            .map(|e| match e {
                Expr::Symbol(s) => s.clone(),
                _ => String::new(),
            })
            .collect())
    } else {
        Ok(vec![])
    }
}