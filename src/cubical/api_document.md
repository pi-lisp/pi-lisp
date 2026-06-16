# Cubical Type Theory — API Reference

A Rust port of a small cubical type theory checker (originally written in Haskell).
The crate is organized as seven modules under `crate::cubical`, layered roughly
bottom-to-top as shown below.

```
interval   (no internal deps)
   ↑
syntax     (interval)
   ↑
eval       (interval, syntax)
   ↑
equality   (interval, syntax, eval)
   ↑
typechecker (interval, syntax, eval, equality)
   ↑
env        (syntax, typechecker)
```

`mod.rs` simply re-exports all six implementation modules:

```rust
pub mod interval;
pub mod syntax;
pub mod eval;
pub mod equality;
pub mod typechecker;
pub mod env;
```

---

## Module: `interval`

Implements the De Morgan interval algebra used by cubical type theory (face
formulas, meet/join/negation) and its normal form.

### Types

| Type | Description |
|---|---|
| `I` | Interval expression syntax: `I0`, `I1`, `IVar(i32)`, `Meet`, `Join`, `Neg`. Implements `Display`. |
| `Literal` | A DNF literal: `Pos(i32)` (`iₙ`) or `NegVar(i32)` (`¬iₙ`). Implements `Display`, `Ord`. |
| `DNF` | `{ cubes: BTreeSet<BTreeSet<Literal>> }` — disjunctive normal form, a set of cubes (conjunctions of literals). Implements `Display`. |

### Functions

```rust
pub fn dnf_top() -> DNF
```
The top element `⊤` — a single empty cube, always true.

```rust
pub fn dnf_bot() -> DNF
```
The bottom element `⊥` — no cubes, always false.

```rust
pub fn eval_interval(i: &I) -> DNF
```
Evaluates an interval expression to its DNF, recursively distributing
meet/join/negation.

```rust
pub fn dnf_join(a: &DNF, b: &DNF) -> DNF
```
Disjunction: union of cube sets, with absorption simplification.

```rust
pub fn dnf_meet(a: &DNF, b: &DNF) -> DNF
```
Conjunction: pairwise union of cubes from each side (cartesian product),
simplified.

```rust
pub fn dnf_neg(d: &DNF) -> DNF
```
Negation via De Morgan distribution over the DNF.

> Internal helpers `simplify`, `neg_cube`, and `neg_lit` are private to this module.

---

## Module: `syntax`

Defines the core `Term` AST, de Bruijn-indexed shifting/substitution, beta
reduction, and pretty printing.

### Types

```rust
pub type Name  = String;
pub type Level = i32;
```

```rust
pub enum Term {
    TVar(i32),
    TApp(Box<Term>, Box<Term>),
    TAbs(Name, Box<Term>),
    TUniv(Level),
    TIntervalTy,
    TPi(Name, Box<Term>, Box<Term>),
    TInterval(I),
    TCube(DNF),
    TPath(Box<Term>, Box<Term>, Box<Term>),
    PLam(Name, Box<Term>),
    PApp(Box<Term>, Box<Term>),
    THComp(Box<Term>, Box<Term>, Box<Term>, Box<Term>),
    TEquiv(Box<Term>, Box<Term>),
    TMkEquiv(Box<Term>, Box<Term>, Box<Term>, Box<Term>, Box<Term>, Box<Term>),
    TEquivFwd(Box<Term>, Box<Term>),
    TUa(Box<Term>),
    TTransport(Box<Term>, Box<Term>),
    TGlue(Box<Term>, Box<Term>, Box<Term>),
    TGlueElem(Box<Term>, Box<Term>, Box<Term>),
    TUnglue(Box<Term>, Box<Term>, Box<Term>),
    TSigma(Name, Box<Term>, Box<Term>),
    TPair(Box<Term>, Box<Term>),
    TFst(Box<Term>),
    TSnd(Box<Term>),
}
```
The full term language: variables, functions (`TAbs`/`TApp`/`TPi`), universes,
the interval type and its terms, path types and path-lambdas, homogeneous
composition (`THComp`), equivalences (`TEquiv`/`TMkEquiv`/`TEquivFwd`),
univalence (`TUa`), `transport`, `Glue` types, and dependent pairs
(`TSigma`/`TPair`/`TFst`/`TSnd`). Derives `Debug`, `Clone`, `PartialEq`, `Eq`.

### Functions

```rust
pub fn show_term(env: &[Name], t: &Term) -> String
```
Pretty-prints `t` using `env` to resolve variable names for bound indices
(falls back to `#i` for out-of-range indices). `Term` also implements
`Display` via `show_term(&[], self)`.

```rust
pub fn shift(d: i32, c: i32, term: &Term) -> Term
```
Increments every free de Bruijn index `>= c` in `term` by `d`. Recurses
through binders, bumping the cutoff `c` by 1 each time a binder is crossed
(`TAbs`, `TPi`, `PLam`, `TSigma`).

```rust
pub fn subst(j: i32, s: &Term, term: &Term) -> Term
```
Substitutes `s` for de Bruijn index `j` inside `term`. Shifts `s` by 1 (and
bumps `j`) when descending under a binder, to keep indices consistent.

```rust
pub fn beta(body: &Term, arg: &Term) -> Term
```
Beta-reduces an abstraction body against `arg`: shifts `arg` up, substitutes
it for index 0, then shifts the result down by 1. Used for `TApp`/`PApp`
reduction in `eval` and for instantiating Pi/Path/Sigma codomains in the
typechecker.

---

## Module: `eval`

A normalization-by-evaluation–style evaluator: weak-head/structural reduction
to normal form, including the cubical primitives (`hcomp`, `transport`,
`Glue`, equivalences).

### Functions

```rust
pub fn is_top_dnf(t: &Term) -> bool
pub fn is_bot_dnf(t: &Term) -> bool
```
True iff `t` is `TCube(d)` with `d` equal to `dnf_top()` / `dnf_bot()`
respectively.

```rust
pub fn eval(t: &Term) -> Term
```
The main evaluator. Highlights:
- `TApp`/`PApp`: evaluates both sides; beta-reduces if the function/path side
  is a `TAbs`/`PLam`, otherwise leaves a neutral application.
- `TAbs`, `TPi`, `TPath`, `PLam`: congruence — evaluate under binders/subterms.
- `TInterval(i)` evaluates to `TCube(eval_interval(i))`.
- `THComp`: if `phi` evaluates to `⊤`, reduces to the tube at `I1`; if `⊥`,
  reduces to `base`; otherwise stays stuck in normal form.
- `TEquivFwd`: applies the equivalence's forward function when the
  equivalence evaluates to `TMkEquiv`.
- `TTransport`: delegates to the internal `eval_transport` (below).
- `TGlue`/`TGlueElem`/`TUnglue`: reduce on the degenerate faces `phi = ⊤` or
  `phi = ⊥`; otherwise stay stuck.
- `TSigma`/`TPair`: congruence; `TFst`/`TSnd` reduce a literal `TPair`, else
  stay stuck.
- Everything else (atoms) returns `t.clone()`.

```rust
pub fn equiv_dom(t: &Term) -> Term
```
Extracts the domain type `A` from an equivalence term (`TMkEquiv` or
`TEquiv`); returns `t` unchanged for anything else.

### Internal helpers (private)

- `syntactic_eq(a, b)` — raw `Term` equality, used only for the trivial-path
  check inside transport (no eta).
- `eval_transport(p_, x_)` — implements `transport` along an evaluated path
  `p_`, case-split on the path's endpoint shape:
  - `TUa(e)` → delegates to `equivFwd e x`.
  - constant path (`b0 == b1`) → identity.
  - `Pi` (non-dependent codomain) → builds a transported function.
  - `Path` → builds a transported path (commutes transport with `PApp`).
  - `Sigma` → transports the first component, then the second along a
    fiberwise `fill`.
  - `Glue` with degenerate `phi` → transports along the underlying type or
    domain; general `Glue` is stuck.
  - anything else → stuck, returned as a neutral `TTransport`.

---

## Module: `equality`

Fuel-bounded definitional/eta equality checking on `Term`s, plus path
boundary reduction used during type checking.

### Types

```rust
pub type Ctx = Vec<(Name, Term)>;
```
A typing context: de Bruijn-indexed name/type pairs, innermost first.

```rust
pub enum EtaResult { Equal, NotEqual, Exhausted }
```
Three-valued verdict of an eta-equality check. `Exhausted` means the fuel ran
out before a verdict was reached (genuinely inconclusive, not necessarily
unequal).

### Functions

```rust
pub fn term_size(t: &Term) -> usize
```
Structural node count of `t`, used to derive starting fuel.

```rust
pub fn initial_fuel(t1: &Term, t2: &Term) -> usize
```
`max(term_size(t1) + term_size(t2), 16)` — the floor of 16 guarantees small
terms get reasonable headroom for eta-expansion steps.

```rust
pub fn and_result(a: EtaResult, b: EtaResult) -> EtaResult
```
Conjunctive combinator: `Equal` is the identity, `Exhausted` is infectious,
and `NotEqual` wins over `Equal` but loses to `Exhausted`.

```rust
pub fn definitionally_equal(t1: &Term, t2: &Term) -> bool
pub fn definitionally_equal_ctx(ctx: &Ctx, t1: &Term, t2: &Term) -> bool
```
Evaluate both terms, then short-circuit on raw equality or fall back to
`eta_eq` with fuel derived from `initial_fuel`. The `_ctx` variant threads a
typing context through eta-expansion (needed to infer domain types for
neutral terms).

```rust
pub fn definitionally_equal_ctx_r(ctx: &Ctx, t1: &Term, t2: &Term) -> EtaResult
```
Like `definitionally_equal_ctx` but returns the full `EtaResult` so callers
(the typechecker) can distinguish a genuine mismatch from fuel exhaustion and
raise the appropriate `TypeError`.

```rust
pub fn reduce_papp_by_type(ctx: &Ctx, p: &Term, r: &Term) -> Option<Term>
```
If `p : Path A u v` and `r` evaluates to `I0`/`I1` (or a top/bottom DNF
cube), returns the corresponding endpoint `u`/`v`; otherwise `None`.

```rust
pub fn infer_lam_dom(ctx: &Ctx, neutral: &Term) -> Option<Term>
```
Infers the Pi-domain type of a neutral term from `ctx`, used to pick the type
of the fresh variable introduced when eta-expanding a neutral term against a
lambda. `None` if the type can't be determined.

```rust
pub fn eta_eq(fuel: usize, ctx: &Ctx, t1: &Term, t2: &Term) -> EtaResult
```
The core fuel-bounded equality algorithm. Checks, in order:
1. `fuel == 0` → `Exhausted`. Raw equality → `Equal`.
2. Path-boundary reduction via `reduce_papp_by_type` on either side
   (consumes 1 fuel).
3. Lambda eta: both-lambda congruence, or eta-expansion when exactly one
   side is a `TAbs` and the other neutral (consumes 1 fuel; `Exhausted` if
   the domain type can't be inferred).
4. Path-lambda eta: the same pattern for `PLam`.
5. Structural congruence on neutral application spines (`TApp`/`PApp`),
   type formers (`TPi`/`TPath`/`TSigma`), and pairs — no fuel consumed.
6. Sigma eta: one side a `TPair`, other neutral — compares against
   `TFst`/`TSnd` projections (consumes 1 fuel).
7. Projection congruence on `TFst`/`TFst` and `TSnd`/`TSnd` spines — no fuel
   consumed.
8. Otherwise, `NotEqual`.

### Internal helpers (private)

`infer_ty` and `infer_neutral_ty` are near-identical private helpers that
infer the type of a `TVar` or a `TApp` spine from `ctx`, used respectively by
`reduce_papp_by_type` and `infer_lam_dom`.

---

## Module: `typechecker`

Bidirectional type checking (`infer` / `check`) for the full term language,
plus the `TypeError` type and assorted "require" helpers.

### Types

```rust
pub type Ctx = Vec<(Name, Term)>;
```
Same shape as `equality::Ctx` (an independent definition in this module).

```rust
pub enum TypeError {
    UnboundVariable(Name),
    TypeMismatch(Term, Term),
    ExpectedPi(Term),
    ExpectedPath(Term),
    ExpectedUniverse(Term),
    ExpectedEquiv(Term),
    ExpectedSigma(Term),
    NotAnInterval(Term),
    CannotInfer(Term),
    EtaFuelExhausted(Term, Term),
    Other(String),
}
```
Implements `Display` with a multi-line, human-readable rendering for each
variant (e.g. `TypeMismatch` shows expected vs. got; `EtaFuelExhausted`
explains that the terms may be equal but too deep to decide automatically).

### Context helpers

```rust
pub fn extend_ctx(x: Name, ty: Term, ctx: &Ctx) -> Ctx
```
Prepends `(x, ty)` to `ctx` (innermost-first ordering).

```rust
pub fn lookup_ctx(i: i32, ctx: &Ctx) -> Result<Term, TypeError>
```
Looks up de Bruijn index `i`, shifting the stored type up by `i + 1` to
account for binders crossed since it was added. Errors with
`UnboundVariable` if `i` is out of range.

### "Require" helpers

```rust
pub fn require_equal(ctx: &Ctx, expected: &Term, got: &Term) -> Result<(), TypeError>
```
Wraps `definitionally_equal_ctx_r`, turning `NotEqual`/`Exhausted` into
`TypeMismatch` / `EtaFuelExhausted` errors.

```rust
pub fn require_equal_endpt(ctx: &Ctx, expected: &Term, got: &Term) -> Result<(), TypeError>
```
Like `require_equal`, but for path/face-endpoint checks: on mismatch it
builds a richer `Other` error including context depth, names, and both the
pretty-printed and raw renderings of each side.

```rust
pub fn require_universe(ctx: &Ctx, t: &Term) -> Result<Level, TypeError>
```
Infers `t`'s type and requires it to evaluate to `TUniv(n)`, returning `n`.

```rust
pub fn check_interval(ctx: &Ctx, t: &Term) -> Result<(), TypeError>
```
Succeeds immediately for `TInterval`/`TCube`; otherwise infers `t`'s type and
requires it to be `TIntervalTy`.

```rust
pub fn require_equiv(ctx: &Ctx, t: &Term) -> Result<(Term, Term), TypeError>
```
Infers `t`'s type and requires it to evaluate to `TEquiv(a, b)`, returning
`(eval(a), eval(b))`.

### Face restriction

```rust
pub fn apply_literal(lit: &Literal, t: &Term) -> Term
```
Applies a single DNF literal as a face substitution on `t`: `Pos(n)` sets
`iₙ = I1`, `NegVar(n)` sets `iₙ = I0`. Recurses structurally through interval
expressions, `TCube` DNFs (re-normalizing after substitution), and all
`Term` constructors, re-evaluating wherever a primitive could now reduce
(`TApp`, `PApp`, `THComp`, `TEquivFwd`, `TTransport`, `TGlue`, `TGlueElem`,
`TUnglue`, `TFst`, `TSnd`).

### Internal helper (private)

```rust
fn check_faces(ctx: &Ctx, phi: &Term, tube_at0: &Term, base: &Term) -> Result<(), TypeError>
```
For `hcomp`: checks that `tube@0 ≡ base` holds on every face (cube) of
`phi`'s DNF, by applying each cube's literals to both sides via
`apply_literal` and comparing with `require_equal_endpt`. Falls back to a
direct comparison if `phi` isn't in `TCube` form.

### Inference and checking

```rust
pub fn infer(ctx: &Ctx, t: &Term) -> Result<Term, TypeError>
```
Synthesizes a type for `t`. Handles every term former except introduction
forms that need a target type to check against (`TAbs`, `PLam`, bare
`TPair`, and `TGlueElem`/`THComp`'s tube case) — those return
`CannotInfer`/are routed through `check`. Notable cases:
- `TApp` / `PApp`: infer the function/path type, require `Pi`/`Path`,
  `check` the argument, return the instantiated codomain via `beta`.
- `TPi` / `TSigma`: both components must be universes; result is the level
  max.
- `TPath`: domain must be a universe; endpoints checked against the
  (possibly path-lambda-derived) endpoint types.
- `TMkEquiv`: checks `f : A → B`, `g : B → A`, and the `eta`/`eps` round-trip
  proofs against their expected Pi/Path types; returns `TEquiv(A, B)`.
- `TEquivFwd`, `TUa`, `TTransport`: as described under `eval`'s transport
  section, with corresponding type-level checks.
- `TGlue` / `TUnglue`: case-split on `phi`'s degenerate faces; otherwise
  combine universe levels or require a `Glue`-typed argument.
- `TGlueElem`: can only be inferred when `phi` is degenerately `⊤`/`⊥`;
  otherwise `CannotInfer`.
- `TFst` / `TSnd`: require a `Sigma`-typed argument; `TSnd`'s result is
  instantiated with `beta(b_ty, TFst(p))`.
- `THComp`: checks `A` is a universe, `base : A`, and — depending on whether
  the tube is a `PLam` or a generic `Path`-typed term — that the tube agrees
  with `base` on every face of `phi` via `check_faces`.

```rust
pub fn check(ctx: &Ctx, t: &Term, ty: &Term) -> Result<(), TypeError>
```
Checks `t` against `ty`. Handles the introduction forms directly
(`TAbs` against `TPi`, `PLam` against `TPath` — including endpoint checks via
`require_equal_endpt`, `TGlueElem` against `TGlue`, `TPair` against
`TSigma`), and falls through to `infer` + `require_equal` for everything
else.

### Top-level convenience

```rust
pub fn infer_closed(t: &Term) -> Result<Term, TypeError>
pub fn check_closed(t: &Term, ty: &Term) -> Result<(), TypeError>
```
`infer`/`check` against an empty context.

```rust
pub fn report_infer(label: &str, t: &Term)
pub fn report_check(label: &str, t: &Term, ty: &Term)
```
Convenience printers for examples/tests: run `infer_closed`/`check_closed`
and print a ✓/✗ line with the result or error to stdout.

### Internal trait extension (private)

`impl EtaResult { fn is_equal(&self) -> bool }` — a private convenience used
within this module to test `definitionally_equal_ctx_r(..) == EtaResult::Equal`.

---

## Module: `env`

A flat global environment of named definitions, with substitution-based
inlining (rather than wrapping terms in extra `TApp`/`TAbs` layers) and
env-aware wrappers around `infer`/`check`.

### Types

```rust
pub type GlobalEnv = Vec<(Name, Term, Term)>;
```
A list of `(name, type, value)` triples, stored **most-recent first**.

### Functions

```rust
pub fn global_ctx(genv: &GlobalEnv) -> Ctx
```
Builds a `Ctx` from `genv` by reversing it (so variables are innermost-first)
and dropping each definition's value, keeping only `(name, type)`.

```rust
pub fn apply_globals(genv: &GlobalEnv, t: &Term) -> Term
```
Inlines every global definition into `t` via de Bruijn substitution. Globals
occupy indices `0..n-1` with the most-recent global at index 0 and the
oldest at index `n-1`; substitution proceeds outermost (highest index) first
so that earlier substitutions don't disturb later indices, and the term is
shifted down by one after each substitution to close the resulting gap.

```rust
pub fn infer_with_env(genv: &GlobalEnv, t: &Term) -> Result<Term, TypeError>
pub fn check_with_env(genv: &GlobalEnv, t: &Term, ty: &Term) -> Result<(), TypeError>
```
`infer`/`check` against the `Ctx` derived from `genv` via `global_ctx`.

### Internal helper (private)

```rust
fn subst_global(k: i32, v: &Term, body: &Term) -> Term
```
Substitutes the global at de Bruijn index `k` with (a correctly shifted)
`v` into `body`, then shifts the result down by 1.

---

## Quick index of public functions by module

| Module | Public functions |
|---|---|
| `interval` | `dnf_top`, `dnf_bot`, `eval_interval`, `dnf_join`, `dnf_meet`, `dnf_neg` |
| `syntax` | `show_term`, `shift`, `subst`, `beta` |
| `eval` | `is_top_dnf`, `is_bot_dnf`, `eval`, `equiv_dom` |
| `equality` | `term_size`, `initial_fuel`, `and_result`, `definitionally_equal`, `definitionally_equal_ctx`, `definitionally_equal_ctx_r`, `reduce_papp_by_type`, `infer_lam_dom`, `eta_eq` |
| `typechecker` | `extend_ctx`, `lookup_ctx`, `require_equal`, `require_equal_endpt`, `require_universe`, `check_interval`, `require_equiv`, `apply_literal`, `infer`, `check`, `infer_closed`, `check_closed`, `report_infer`, `report_check` |
| `env` | `global_ctx`, `apply_globals`, `infer_with_env`, `check_with_env` |