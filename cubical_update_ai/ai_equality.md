Now let me read the actual current files to understand exactly what's in them:Now let me read syntax.rs to see the new variants:Now I have everything I need. The changes to `equality.rs` are:

1. **`term_size`** — needs cases for `TData`, `TCon`, `TPCon`, `TElim`
2. **`eta_eq`** — needs congruence for `TCon`, `TPCon`, `TElim`, and `TData` (which are either atoms or have straightforward structural equality)

Let me write the updated file:Now add the `eta_eq` cases. The right place is before the final `NotEqual` return — after the projection congruence block:Now copy to outputs and present:Here's what changed and why:

**`term_size`** — four new arms added at the end of the match:
- `TData(_)` → 1 (atom, like `TUniv`)
- `TCon(_, _, args)` → 1 + sum of arg sizes
- `TPCon(_, _, args, r)` → 1 + args + size of `r`
- `TElim(motive, cases, scrut)` → 1 + motive + sum of case *body* sizes + scrutinee. The case binder names are just strings and don't contribute meaningfully to fuel, so I only count bodies — consistent with how `TAbs` counts its body but not its `Name`.

**`eta_eq`** — four cases added just before the final `NotEqual`, after the projection congruence block:

- **`TData`**: no arm needed — if both sides are `TData("X")` they're already caught by `t1 == t2` at the top; different names fall through to `NotEqual`. This matches how `TUniv(n)` is handled.

- **`TCon` congruence**: checks datatype name, constructor name, and arity match first (short-circuit to `NotEqual` if not), then folds `and_result` over paired args. No fuel consumed — purely structural, same as `TApp`.

- **`TPCon` congruence**: same, plus checks the interval argument `r` after the ordinary args.

- **`TElim` congruence**: this only fires when both sides are *stuck* neutral eliminators (since `eval` would have reduced any redex). It checks case count, then for each paired case verifies constructor name and binder count match, builds an extended context for the case body by pushing binders innermost-first (last binder → index 0, matching the codebase convention), and recurses on the bodies. Then checks motive and scrutinee. No fuel consumed — structural.

One subtle thing in the `TElim` case: the binder types are stored in `ConSig`/`PConSig` in the datatype schema, not in `ElimCase` itself, so I use `TUniv(0)` as a placeholder type for the fresh context variables — the same fallback `eta_eq` already uses when `infer_lam_dom` returns `None` on a lambda. This is conservative but correct for the structural congruence use case (we're comparing bodies that are already in the same stuck neutral position, not doing type-directed eta-expansion through them).