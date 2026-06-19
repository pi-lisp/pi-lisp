Yes. Let me read the relevant parts of eval.rs carefully before touching anything:Good. Now I need to think carefully about what the correct reduction rules are before writing any code.

**Transport `⟨i⟩ TData(d)` x:** `TData` is a constant type family — it doesn't depend on `i`. So `b0 == b1 == TData(d)`, which means `syntactic_eq(&b0, &b1)` is already true, and transport returns `x_` unchanged. This case is **already handled** by the trivial-path check. No new code needed for transport over a constant HIT type.

**Transport `⟨i⟩ (elim over TData that varies with i)` x:** This is the hard case — transport in a motive that's indexed by a HIT element that varies with `i`. This is genuinely undecidable in general without knowing the specific HIT's transport rules. **Leave stuck.**

**HComp for `TData(d)`:** This is the interesting case. When `a_ty` evaluates to `TData(d)` and `phi` is non-trivial, we have a genuine open composition problem in a HIT. The canonical cubical approach is to reduce it via the **hub-and-spoke** or **flattening** construction — but this requires knowing the specific HIT's higher structure and is highly non-trivial to implement generally. The practical approach used in most cubical provers (agda/cubical, cooltt) is: **leave stuck unless `phi` is top or bottom** (which is already handled by the existing `THComp` top/bottom cases). So no new case is needed in `THComp` either for the general HIT case.

However — there's one specific hcomp reduction that *is* straightforward and needed: **hcomp in `TData(d)` when `x_` (the base) is a `TCon`** and the tube is compatible. Actually, even this requires knowing the HIT's constructors to know how to fill. This is HIT-specific and can't be done generically without the datatype schema at eval time (eval currently has no access to `&[Datatype]`).

So the honest answer: **transport and hcomp for HITs are both already correctly handled by the existing trivial/degenerate cases** — transport over a constant `TData` returns the element unchanged (caught by `syntactic_eq`), and hcomp with `phi = ⊥` returns the base and with `phi = ⊤` returns the tube at `I1`. The general non-degenerate cases must stay stuck for now.

Let me verify the `syntactic_eq` catch actually fires correctly:

For `transport (⟨i⟩ S¹) base`: `body = TData("S1")` (constant), `b0 = eval(TData("S1")) = TData("S1")`, `b1 = TData("S1")`, `syntactic_eq(b0, b1)` = true → returns `x_`. ✓

The only real gap is `TData` in `apply_literal`'s `go` in `eval.rs` — but that's in `typechecker.rs`'s copy of `apply_literal`, not in `eval.rs`. Let me check if `eval.rs` itself has an `apply_literal`:`eval.rs` has no `apply_literal` — that only lives in `typechecker.rs`. And the inductive cases in `eval.rs` are already written. So there is genuinely nothing new to add to `eval.rs` for transport/hcomp.

The summary: **no changes to `eval.rs` are needed.** Here's why each case is already covered:

- **`transport (⟨i⟩ TData(d)) x`** — `TData` doesn't mention `i`, so `b0 == b1`, caught by the existing `syntactic_eq` trivial-path guard → returns `x` unchanged. ✓

- **`transport (⟨i⟩ <non-trivial HIT-indexed type>) x`** — falls through to the `_ => stuck` arm. This is correct: general transport in a HIT-indexed type family requires HIT-specific reduction rules (like the `S¹` induction principle) that can't be expressed generically without the datatype schema at eval time. Implementing these would require threading `&[Datatype]` into `eval`, which is a much larger refactor.

- **`hcomp TData(d) ⊥ tube base`** → returns `base`. ✓  
  **`hcomp TData(d) ⊤ tube base`** → returns `tube @ I1`. ✓  
  **`hcomp TData(d) phi tube base`** (non-degenerate) — falls through to the congruence-stuck arm. This is also correct for now: non-degenerate composition in a HIT requires knowing the HIT's specific composition structure (for `S¹` this involves the `loop` constructor; there's no generic rule).

When you later want these to reduce — say, to support `S¹` path concatenation or the `J` eliminator working on HIT elements — the right approach is to thread `&[Datatype]` through `eval` and add HIT-specific cases in `eval_transport` keyed on the datatype name. That's a significant refactor and should wait until the parser and basic tests are working so you can actually exercise those code paths.