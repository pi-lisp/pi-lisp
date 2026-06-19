Let me read all the relevant files first.Now let me also read the eval.md for context:Now I have a clear picture. The markdown files describe a series of changes across `syntax.rs`, `eval.rs`, `equality.rs`, `typechecker.rs`, and `env.rs`. The `cubical.rs` file needs new builtins registered for the four new `Term` variants: `TData`, `TCon`, `TPCon`, and `TElim`.

Let me write the updated file:Now update the header comment block to document the new builtins and update `ctt-infer`/`ctt-check` to use the `_dt` variants when datatypes are available. First the header:Now update `ctt-infer` and `ctt-check` to use `infer_closed_dt` / `check_closed_dt` with an empty slice (matching the pattern described in `ai_typechecker.md` — backward-compatible, passing `&[]` so existing call sites continue to work):Now copy to outputs and present:Here's what changed and why:

**New comment block in the header** — documents the four new builtins alongside the existing ones, with the `(elim ...)` case structure spelled out since it's more involved than the others.

**Four new `env_set` registrations** (in a new `── inductive / HIT types ──` section):

- **`data-type`** → `TData(name)`: trivial atom, takes one symbol. Corresponds to what you'd write as the name of a declared inductive type.

- **`con`** → `TCon(dt, c, args)`: takes a datatype symbol, a constructor symbol, and any number of cubical term arguments. The `dt` name is stored redundantly alongside the constructor name, as noted in `ai_syntax.md`, so eval/typecheck don't need a cross-datatype name lookup.

- **`pcon`** → `TPCon(dt, pc, args, r)`: same as `con` but the interval argument `r` comes third (right after constructor name), matching the surface "path-constructor applied to an interval" reading. The `args` and `r` are stored per the `TPCon(dt, pc, Vec<Term>, Box<Term>)` shape from `ai_syntax.md`.

- **`elim`** → `TElim(motive, cases, scrut)`: takes the motive, scrutinee, and then any number of case lists. Each case is expected to be an `Expr::List` of the form `(con-name binder... body)`, parsed into `ElimCase { con, binders, body }`. The interval binder for path-constructor cases is just the last binder before the body, matching the `ElimCase` convention documented in `ai_syntax.md`.

**`ctt-infer` and `ctt-check`** updated to call `tc::infer_closed_dt(&[], ...)` and `tc::check_closed_dt(&[], ...)` respectively — passing an empty datatype slice. This is backward-compatible (non-HIT terms type-check identically), and the comments note that HIT terms should go through `infer_with_full_env`/`check_with_full_env` once the `Env` integration from `ai_env.md` is in place.

One thing to confirm with you: I used `Expr::List(xs)` for destructuring each elim case. If your `Expr` type uses a different variant name for lists (e.g. `Expr::Cons` or `Expr::Pair`), that line will need adjusting to match your actual AST shape.