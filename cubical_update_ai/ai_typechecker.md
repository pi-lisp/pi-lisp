Let me read the typechecker first:Good. Now I have the full picture. The typechecker needs:

1. `apply_literal`'s inner `go` function needs cases for `TData`, `TCon`, `TPCon`, `TElim`
2. `infer` needs cases for `TData`, `TCon`, `TPCon`, `TElim`
3. `check` gets `TCon`/`TPCon` as new intro forms (the existing fallthrough handles them, but it's worth being explicit about what the checker needs from a `Datatype` schema — except we need to decide: does the typechecker take a `&[Datatype]` or is it looked up elsewhere?)

Let me look at what info `infer` needs — for `TData` it's a universe, for `TCon`/`TPCon` it needs argument types and return type, for `TElim` it needs the motive applied to the scrutinee. All of this requires access to the `Datatype` schema. The current typechecker has no global env thread — `infer` takes only `ctx`. I need to understand how `GlobalEnv` and `Datatype` declarations are intended to connect.

Looking at `env.rs`: it stores `(Name, Term, Term)` triples. The `Datatype` schema is separate. The cleanest approach matching this codebase's style is to thread a `&[Datatype]` alongside `ctx` — but that would change every `infer`/`check` signature, which is very invasive.

The alternative used in many cubical implementations: store `TData` as a term that is simply in the global `Ctx` (with type `TUniv(n)`), and store constructors as global definitions too. But that can't express the eliminator typing without schema access.

Looking more carefully — the `ai_syntax.md` notes don't prescribe a specific threading strategy. The least invasive approach that matches the existing architecture: add a thread-local or a separate `infer_with_dt` / `check_with_dt` that passes `&[Datatype]` alongside `ctx`, mirroring how `env.rs` has `infer_with_env`. Let me write it that way — new `infer_dt` / `check_dt` functions that shadow `infer`/`check` and take an extra `dts: &[Datatype]`, with the existing `infer`/`check` delegating to them with an empty slice. This is zero breaking-change.Now add new `TypeError` variants:Add their `Display` arms:Now add `apply_literal`'s `go` cases for the new variants. The `go` inner function is in a block — I need to add them before the catch-all `other => other.clone()`:Now add the `infer`/`check` cases. First I need to restructure to thread `dts`. The cleanest zero-breakage approach: change the existing `infer` and `check` to take `dts: &[Datatype]` and update all internal recursive calls, then make `infer_closed`/`check_closed` pass `&[]`. Let me do this carefully:Now I need to update all the recursive `infer(ctx, ...)` calls inside `infer_dt` to use `infer_dt(dts, ctx, ...)`, and similarly for `check`. But rather than doing that surgically (very error-prone with so many call sites), let me use a different approach: keep `infer` and `check` as-is internally, and make them call new `infer_dt`/`check_dt` only for the new cases. Actually the cleanest real-world approach: the new cases need `dts`, so I'll add `infer_dt` / `check_dt` as the real implementations, have `infer`/`check` call them with `&[]`, and make the new inductive cases in `infer_dt` call `infer_dt`/`check_dt` recursively. The existing cases don't have inductive sub-terms so they don't need `dts` — and the recursive calls they make will go through `infer` which passes `&[]` (fine since those sub-terms don't contain `TData`/`TCon`/etc. in the non-inductive old cases).

Wait — that's wrong. An eliminator's motive can contain TApp of ordinary terms. The recursion inside the old cases doesn't need dts. Let me revert the infer split and instead just add the new cases at the end of the existing `infer` match:Now I need to replace the `infer_via_reduction` helper call pattern — it calls `infer(ctx, &reduced)` internally. Let me check it:This calls `infer` (no dts) which is fine — `infer_via_reduction` is only used for neutral spine reduction of `TApp`/`PApp`/`TFst`/`TSnd`, none of which produce inductive terms that need dts. Now I need to close the `infer_dt` function and add the new cases to it. Let me find where the closing brace of `infer` (now `infer_dt`) is, and add the new cases before it:Now I'll add the inductive cases before the closing `}` of `infer_dt`:Now add `check_dt` and update `check` to delegate to it:Now update the top-level helpers and add `infer_dt`/`check_dt` variants:Now let me do a quick sanity-check read of the key new sections:Looks good. Now I notice there's one issue with the `motive_ty` variable in `TElim` — I construct it but never use it (I infer the motive's type instead). Let me clean up that dead code:Now copy to outputs:Here's a summary of every change and the reasoning behind each decision:

## What changed

**Imports:** Added `ElimCase` and `Datatype` to the `syntax` import.

**`TypeError` — 6 new variants:**
- `UnknownDatatype(Name)` — `TData`/`TCon`/etc. reference a datatype not in `dts`
- `UnknownConstructor(Name, Name)` — constructor name not found in that datatype's sig
- `WrongNumberOfArgs { con, expected, got }` — arg count mismatch at a constructor site
- `BadElimCase { con, msg }` — wrong binder count in an eliminator arm
- `MissingCase(Name)` — eliminator is missing an arm for a constructor
- `ExpectedData(Term)` — scrutinee of `TElim` isn't a `TData`

**`apply_literal`'s inner `go` function:** Added cases for `TData` (atom, clone), `TCon`/`TPCon` (recurse into all sub-terms and re-eval), and `TElim` (recurse into motive, all case bodies, and scrutinee, then re-eval). These are required for correct face-restriction of HIT terms under a face of the interval — for instance `loop @ i` restricted to `i=0` must propagate through the body of a `TElim` that eliminates it.

**`infer` → `infer_dt`, `check` → `check_dt`:** The public `infer`/`check` now delegate to `infer_dt`/`check_dt` with `dts = &[]`, preserving full backward compatibility. All internal recursive calls within `check_dt` use `check_dt`/`infer_dt` so datatypes thread correctly through deep checking. `infer_via_reduction` stays calling bare `infer` (fine, since it only fires on neutral spines of non-inductive terms).

**New `infer_dt` cases:**

*`TData(d)`* — looks up `d` in `dts`, returns `TUniv(0)`. Universe level is hardcoded to 0 for now; widen later if needed.

*`TCon(d, c, args)`* — looks up the `ConSig`, checks arity, then iterates the telescope: at position `k`, substitutes `checked_args[0..k-1]` (innermost-first via `iter().rev().fold(... beta)`) into `arg_tys[k]` to instantiate any dependent types. Returns `TData(d)`.

*`TPCon(d, pc, args, r)`* — same telescope check for ordinary args, then `check_interval` on `r`. Computes `face0`/`face1` by beta-substituting all checked args into `sig.face0`/`sig.face1` (which live in the arity-arg scope). Returns `TPath(TData(d), face0[args], face1[args])`.

*`TElim(motive, cases, scrut)`* — infers scrutinee type, requires it's `TData(d)`, infers motive's type and requires it's `Π(_:TData(d)).U_n`. Then checks every ordinary-constructor case: builds an extended context by pushing binders incrementally (substituting already-bound args into later `arg_tys` as it goes, same telescope pattern), then checks the case body at type `motive (TCon d c vars)` where `vars` are the de Bruijn indices for those binders. For path-constructor cases: extends context with `arity` ordinary binders then the interval binder last (index 0), and checks the body (PLam-shaped) at `Path (PLam i. motive (pcon args i)) (motive face0) (motive face1)`. Returns `motive scrut`.

**`check_dt`:** Mirrors the existing `check` but recurses via `check_dt`/`infer_dt`. Adds an explicit arm for `TCon`/`TPCon` that delegates to `infer_dt` (same as the fallthrough, but explicit to make it clear these are intro forms). 

**New top-level helpers:** `infer_closed_dt` and `check_closed_dt` for calling with a concrete `dts` slice at the top level (e.g. from `env.rs`'s `infer_with_env`).