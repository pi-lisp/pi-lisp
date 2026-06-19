Good — named-binder cases, file-by-file review. Let's start with `syntax.rs`, since everything else builds on it.

## Design for this file

**Schema types** (new):
```rust
pub struct ConSig {
    pub name: Name,
    pub arity: usize,        // number of ordinary term arguments
}

pub struct PConSig {
    pub name: Name,
    pub arity: usize,        // number of ordinary term arguments (before the interval arg)
    pub face0: /* boundary at i=0, as a term-building recipe over the con's own args */ ...
    pub face1: /* boundary at i=1 */ ...
}

pub struct Datatype {
    pub name: Name,
    pub cons: Vec<ConSig>,
    pub pcons: Vec<PConSig>,
}
```

I paused here because the boundary equations (`face0`/`face1`) are the trickiest part: they're terms that mention the path constructor's *own* bound arguments, so they need to be stored as `Term`s under a binder scope matching `arity`, using the same de Bruijn convention as everything else (innermost variable = highest-numbered argument = index 0 at the point the boundary is checked). I'll write that out concretely in the code so it's auditable rather than hand-waved.

**Term additions:**
- `TData(Name)` — 0-ary, like `TUniv`
- `TCon(Name, Name, Vec<Term>)` — `(datatype, constructor, args)`. I include the datatype name redundantly so eval/typecheck don't need a name-only lookup that could collide across datatypes.
- `TPCon(Name, Name, Vec<Term>, Box<Term>)` — `(datatype, pconstructor, args, interval arg)`
- `TElim(Box<Term> /* motive */, Vec<ElimCase>, Box<Term> /* scrutinee */)`

```rust
pub struct ElimCase {
    pub con: Name,
    pub binders: Vec<Name>,   // arg names; for a pcon case, last binder is the interval var
    pub body: Box<Term>,
}
```

I'll implement `shift`/`subst`/`show_term` for all of these now, matching your existing binder-counting conventions exactly (e.g. `TAbs` shifts body at `c+1`; an `ElimCase` body must shift at `c + binders.len()`).

Let me write it.Confirmed — this matches what I had in context. Now let me write the extended `syntax.rs`.

A key design decision for the boundary equations: a path constructor like `loop : (i : I) → S¹` with `loop 0 = base`, `loop 1 = base` is declared with `arity = 0` ordinary args, and its `face0`/`face1` are `Term`s in a scope of exactly `arity` bound variables (the constructor's own ordinary args, NOT including the interval variable — the interval var is fixed to `I0`/`I1` at the boundary, so it's not free in the boundary term). For something like a hypothetical indexed-with-args case `Term::TPCon`, the same convention extends naturally: `arity` ordinary args are in scope, indices `0..arity-1`, innermost-first like every other binder in this codebase.Now `show_term`:I caught myself on a subtlety: in `env2.reverse()` for pretty-printing, that's correct because `binders` is outermost-first but `env`/context conventions elsewhere in this file are innermost-first (e.g. `TAbs`'s `env2 = [x] ++ env`, with `x` becoming index 0). For a multi-binder case, the *last* declared binder must end up at index 0. Reversing `binders` then prepending gives exactly that. Now `shift` and `subst`:Note the motive in `TElim` is itself shifted at `c` (not `c+1`) here, since per the doc comment the motive is "a `TAbs`-shaped term" — meaning its own binder is internal to the `Term::TAbs` node and handled by that case already. Now `subst`:The `shift(n, 0, s)` / `subst(j + n, ...)` pattern here generalizes the existing single-binder pattern (`shift(1,0,s)` / `subst(j+1,...)`) to `n`-deep binders in one step, which is equivalent to applying the single-binder version `n` times and is the standard generalization — consistent with how the rest of the file shifts a substituted value once per crossed binder.

Let's compile-check this in isolation against the rest of the crate (best effort, since I don't have the full crate, but I can at least check the file parses as valid Rust and is internally self-consistent):No Rust toolchain available in this sandbox (network is locked to package registries, not apt for rustc... let me check).Got rustc. Let me build a minimal crate scaffold (`crate::cubical::*` module paths) so I can actually compile-check this file for real, not just eyeball it.This is exactly the expected and useful kind of error — the Rust compiler is enumerating every place downstream that will need a case for the new variants once I add it to the enum. That's `equality.rs:term_size` and `typechecker.rs` (some match around line 313/602) — exactly the files I planned to extend next. Good confirmation that `syntax.rs` itself parses cleanly and the only fallout is the expected exhaustiveness pressure on downstream files.

Let me view `syntax.rs` as it stands now in full, so you can review it properly:This compiles cleanly within `syntax.rs` itself (the two errors above are both in *other* files, as expected, since I haven't touched them yet).

## Summary of what's in this file now

- **`Term`**: 4 new variants — `TData`, `TCon`, `TPCon`, `TElim` — added at the end so existing variant ordering/discriminants in other code (if any) aren't disturbed.
- **`ElimCase`**: named-binder case arm, as you asked for.
- **`ConSig` / `PConSig` / `Datatype`**: the generic `data` declaration schema. `PConSig` carries `face0`/`face1` directly as `Term`s — this is the load-bearing HIT piece, and I documented exactly which variables are in scope there (only the constructor's own ordinary args, not the interval var) since getting that wrong would quietly break every boundary check downstream.
- **`show_term`, `shift`, `subst`**: extended consistently with the file's existing innermost-first / outermost-first conventions. I was deliberate about the one place these conventions could silently diverge — multi-variable `ElimCase` binders — and wrote out the reasoning in comments rather than just asserting it, since this is exactly the kind of thing that compiles fine and is wrong.

**Before I move on to `eval.rs`** (ι-reduction for `TElim`/`TCon`, plus boundary reduction at `I0`/`I1` for `TPCon`), let me know if:

1. The `TPCon` shape (args separate from the single interval arg `r`) matches how you want multi-dimensional path constructors to scale later, or if you'd rather generalize `r` to `Vec<Term>` now (for HITs needing 2D faces, e.g. a torus) even though we don't have a concrete use for it yet.
2. The `ElimCase` "interval binder goes last, body is PLam-shaped" convention reads correctly to you, since it's the one place I made a judgment call rather than copying an existing pattern.