> You are an expert Rust systems programmer. The bytecode VM is working correctly for arithmetic, comparisons, `if`, `quote`, `list`, `begin`, and builtin calls. Now extend the VM to handle `let`, `let*`, and top-level `define` — without breaking any existing functionality.
>
> ## Current state
>
> - `vm/bytecode.rs` — defines `Op`, `Value`, `Chunk`; `Op::StoreVar(String)` already exists
> - `vm/compiler.rs` — `is_compilable` currently returns `false` for any expression containing `define`, `let`, or `let*`; the compiler does not emit `StoreVar` for these forms
> - `vm/machine.rs` — `Op::StoreVar` may or may not be implemented in the dispatch loop; verify and fix if missing
> - All environment mutations go through `env_set(heap, env, name, val)` which writes to `GcHandle { idx: 0 }` (the shared global frame)
>
> ## What I want you to change
>
> ### 1. `vm/compiler.rs` — compile `let`, `let*`, `define`
>
> **`let`**
> Desugar into sequential `StoreVar`s in a new child env frame:
> ```
> (let ((x 1) (y 2)) body)
> →
>   Op::LoadConst(1)
>   Op::StoreVar("x")
>   Op::LoadConst(2)
>   Op::StoreVar("y")
>   [compile body]
> ```
> All bindings are evaluated before any are stored (evaluate all RHS first, then emit all `StoreVar`s).
>
> **`let*`**
> Same as `let` but each binding is stored immediately so later bindings can reference earlier ones:
> ```
> (let* ((x 1) (y (+ x 1))) body)
> →
>   Op::LoadConst(1)
>   Op::StoreVar("x")        ← stored before y's RHS is compiled
>   [compile (+ x 1)]
>   Op::StoreVar("y")
>   [compile body]
> ```
>
> **`define`**
> Emit the RHS, then `StoreVar`:
> ```
> (define name expr)
> →
>   [compile expr]
>   Op::StoreVar("name")
>   Op::LoadConst(())        ← define returns ()
> ```
>
> ### 2. `vm/compiler.rs` — update `is_compilable`
>
> Remove `define`, `let`, and `let*` from the blocklist. They are now compilable.
> Keep blocking: `lambda`, `quasiquote`, `set!`, `letrec`, `CubicalTerm`.
>
> ### 3. `vm/machine.rs` — verify `StoreVar` dispatch
>
> Make sure `Op::StoreVar(name)` in the run loop:
> - pops the top of the stack
> - calls `env_set(self.heap, current_env, name, value_to_expr(val))`
> - does **not** push anything back (the `LoadConst(())` after define handles the return value)
>
> ### 4. Correctness requirements
>
> - `let` bindings must **not** be visible outside the `let` body. Allocate a child env frame with `new_env(heap, Some(current_env))` before the bindings and restore the parent env after the body. Add `Op::PushEnv` and `Op::PopEnv` if needed.
> - `let*` has the same scoping requirement.
> - `define` at top level writes directly into the current frame (no child env).
> - After this change, the full test file must still produce identical output to the tree-walker for all non-cubical expressions.
>
> ### 5. What NOT to change
>
> - Do not implement `lambda` compilation yet
> - Do not change the `CubicalTerm` fallback
> - Do not remove the DEBUG `env_set` log lines yet (still needed for verification)
>
> After implementing, show the updated `is_compilable`, the new compiler branches for `let`/`let*`/`define`, and the `StoreVar`/`PushEnv`/`PopEnv` dispatch in the run loop.