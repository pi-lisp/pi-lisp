Looking at what's complete vs. what's still needed for inductive types to actually work end-to-end:

**Immediately necessary:**

1. **Parser** — nothing can be used without a way to write `data S1 = base | loop : I → S1` and eliminators in surface syntax. This is probably the biggest chunk of work left.

2. **`env.rs` integration** — `GlobalEnv` currently stores `(Name, Term, Term)` triples but has no slot for `Datatype` declarations. You need a way to register datatypes so `infer_dt`/`check_dt` can find them, and a combined env type that carries both definitions and datatype schemas together.

3. **Transport for `TData`** — `eval_transport` in `eval.rs` has no case for `PLam(i. TData(d))`. Right now it falls through to stuck, which means `transport (⟨i⟩ S¹) x` won't reduce. For simple HITs without higher constructors this can sometimes be left stuck, but for univalence to compose correctly with HITs you'll eventually need it.

**Worth doing soon after:**

4. **`hcomp` for `TData`** — same gap in `THComp`: no case for a HIT type, so composition in a HIT is stuck. Needed for the full cubical structure (path concatenation, J, etc. working on HIT elements).

5. **Tests** — a small test suite exercising `S¹` (base, loop, elim, transport along `loop`) and `Bool`/`Nat` (to verify ordinary inductive types work before tackling HIT-specific reductions) would catch the index bugs that are almost certainly lurking in the `TElim` case-body context building.

**The order I'd suggest:** env integration → parser → tests → transport/hcomp for HITs. The env integration is small and unblocks everything else; the parser is the largest piece but makes the system actually usable; tests will immediately reveal whether the typechecker's de Bruijn index arithmetic in the eliminator cases is right (it's the part most likely to be subtly off).

What area are you most interested in tackling first?