## task 2 — VM Machine & Integration

> You are an expert Rust systems programmer. Phase 1 is complete: `vm/bytecode.rs` and `vm/compiler.rs` already exist and compile cleanly. Now implement **phase 2: the stack-based VM and integration layer**.
>
> ## What already exists
>
> - `vm/bytecode.rs` — `Op`, `Value`, `Chunk`, `value_to_expr`, `expr_to_value`
> - `vm/compiler.rs` — `Compiler::compile(expr, heap) -> Result<Chunk, String>`
> - `gc.rs` — `Heap`, `GcHandle`; environment helpers `env_get` / `env_set` / `new_env`
> - `eval.rs` — tree-walking `eval(expr, env, heap)` (must remain untouched as fallback)
>
> Key constraints:
> - Reuse `Heap` / `GcHandle` for all environment frames — no second allocator
> - `Value::Builtin(Rc<dyn Fn(&[Value], &mut Heap) -> Result<Value, String>>)` must be callable directly from the VM dispatch loop
> - Tail-call optimisation is **required**: `TailCall` must reuse the current `CallFrame` instead of pushing a new one
> - No `unsafe` code
>
> ## What I want you to implement
>
> **`vm/machine.rs`**
>
> ```rust
> pub struct CallFrame {
>     chunk: Rc<Chunk>,
>     ip: usize,
>     stack_base: usize,
>     env: GcHandle,
> }
>
> pub struct VM<'h> {
>     stack: Vec<Value>,
>     frames: Vec<CallFrame>,
>     heap: &'h mut Heap,
> }
>
> impl<'h> VM<'h> {
>     pub fn new(heap: &'h mut Heap, env: GcHandle, chunk: Chunk) -> Self;
>     pub fn run(&mut self) -> Result<Value, String>;
> }
> ```
>
> The `run` loop must handle every `Op` variant from `bytecode.rs`.  For `Call` / `TailCall`, resolve the callee from the stack top and either push a new `CallFrame` (normal call) or overwrite the current one (tail call).
>
> **`vm/mod.rs`**
>
> Expose:
> ```rust
> pub fn vm_eval(expr: &Expr, env: GcHandle, heap: &mut Heap) -> Result<Expr, String>
> ```
> This should compile with `Compiler::compile`, run with `VM::run`, and fall back to `crate::eval::eval` on any `Err("uncompilable")` from the compiler.
>
> **Integration in `eval.rs`**
>
> Add a `cfg` feature flag `vm` so that when compiled with `--features vm`, the top-level `eval` function routes through `vm_eval`. When the feature is off, behaviour is identical to today.
>
> After each file, briefly explain any non-obvious design decisions, especially around tail-call frame reuse and builtin dispatch.