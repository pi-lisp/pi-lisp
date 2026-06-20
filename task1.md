> You are an expert Rust systems programmer. The VM is currently slower than the tree-walker because every expression goes through `is_compilable` (a deep tree walk) and `expand_all` + `Compiler::compile` on every execution. The goal is to add caching so these costs are paid at most once per unique expression.
>
> ## Current bottleneck
>
> For every call to `vm_eval(expr, env, heap)`:
> 1. `is_compilable(expr, heap, env)` — deep walks the entire AST, checks env for macros
> 2. `expand_all(expr, env, heap)` — deep walks again, expands macros
> 3. `Compiler::compile(expanded)` — walks again, emits opcodes
> 4. `VM::run(chunk)` — executes
>
> Steps 1–3 are pure overhead for expressions that never change (top-level `define`s, `lambda` bodies, etc.).
>
> ## What I want you to implement
>
> ### 1. `vm/cache.rs` — new file
>
> ```rust
> use std::collections::HashMap;
> use crate::expr::Expr;
> use crate::vm::bytecode::Chunk;
>
> /// Caches compiled Chunks keyed by a stable string representation of the Expr.
> /// Also caches `is_compilable` results to avoid redundant deep walks.
> pub struct CompileCache {
>     /// expr_key → compiled Chunk
>     chunks: HashMap<String, Chunk>,
>     /// expr_key → is_compilable result
>     /// Invalidated when a new macro is defined (see `invalidate_macro_cache`)
>     compilable: HashMap<String, bool>,
> }
>
> impl CompileCache {
>     pub fn new() -> Self { ... }
>
>     /// Stable cache key: use the Debug format of Expr.
>     /// This is not perfect but good enough — two structurally identical
>     /// expressions produce the same key.
>     pub fn key(expr: &Expr) -> String {
>         format!("{:?}", expr)
>     }
>
>     pub fn get_chunk(&self, key: &str) -> Option<&Chunk> { ... }
>     pub fn insert_chunk(&mut self, key: String, chunk: Chunk) { ... }
>
>     pub fn get_compilable(&self, key: &str) -> Option<bool> { ... }
>     pub fn insert_compilable(&mut self, key: String, result: bool) { ... }
>
>     /// Call this whenever a new macro is defined via `defmacro`.
>     /// Clears the `compilable` cache because new macros change what is_compilable returns.
>     /// Does NOT clear the chunk cache (compiled chunks remain valid).
>     pub fn invalidate_compilable(&mut self) { self.compilable.clear(); }
> }
> ```
>
> ### 2. `vm/mod.rs` — thread-local cache
>
> Add a thread-local cache so it persists across `vm_eval` calls without needing to pass it through every function signature:
>
> ```rust
> use std::cell::RefCell;
> use crate::vm::cache::CompileCache;
>
> thread_local! {
>     static CACHE: RefCell<CompileCache> = RefCell::new(CompileCache::new());
> }
> ```
>
> Update `vm_eval` to use the cache:
>
> ```rust
> pub fn vm_eval(expr: &Expr, env: GcHandle, heap: &mut Heap) -> Result<Expr, String> {
>     let key = CompileCache::key(expr);
>
>     // 1. Check compilable cache
>     let compilable = CACHE.with(|c| c.borrow().get_compilable(&key).copied());
>     let compilable = match compilable {
>         Some(v) => v,
>         None => {
>             let result = is_compilable(expr, heap, env);
>             CACHE.with(|c| c.borrow_mut().insert_compilable(key.clone(), result));
>             result
>         }
>     };
>
>     if !compilable {
>         // Detect defmacro: if this expression defines a macro,
>         // invalidate the compilable cache after tree-eval
>         let is_defmacro = matches!(expr, Expr::List(l)
>             if matches!(l.first(), Some(Expr::Symbol(s)) if s == "defmacro"));
>         let result = tree_eval(expr, env, heap)?;
>         if is_defmacro {
>             CACHE.with(|c| c.borrow_mut().invalidate_compilable());
>         }
>         return Ok(result);
>     }
>
>     // 2. Check chunk cache
>     let cached_chunk = CACHE.with(|c| c.borrow().get_chunk(&key).cloned());
>     let chunk = match cached_chunk {
>         Some(chunk) => chunk,
>         None => {
>             // expand_all + compile — paid only once
>             let expanded = expand_all(expr, env, heap)
>                 .unwrap_or_else(|_| expr.clone());
>             let chunk = Compiler::new().compile(&expanded)?;
>             CACHE.with(|c| c.borrow_mut().insert_chunk(key.clone(), chunk.clone()));
>             chunk
>         }
>     };
>
>     // 3. Run the cached chunk
>     let result = VM::new(heap, env, chunk).run()?;
>     Ok(value_to_expr(&result))
> }
> ```
>
> ### 3. Cache invalidation rules
>
> - **`compilable` cache**: invalidated on every `defmacro` because new macros change what `is_compilable` returns for other expressions
> - **`chunk` cache**: never invalidated — compiled chunks are pure functions of the source expression and do not depend on runtime env values. Variable lookups happen at VM runtime via `LoadVar`, not at compile time.
> - **Cache key**: `format!("{:?}", expr)` — simple and correct for this interpreter's use case. Two calls with structurally identical expressions (same AST) share a cache entry.
>
> ### 4. `vm/mod.rs` — expose cache stats (optional but useful)
>
> Add a function for debugging:
> ```rust
> pub fn cache_stats() -> (usize, usize) {
>     CACHE.with(|c| {
>         let c = c.borrow();
>         (c.chunks.len(), c.compilable.len())
>     })
> }
> ```
>
> ### 5. Correctness requirements
>
> - Cached chunks must produce identical results to uncached execution
> - `defmacro` must always invalidate the `compilable` cache, even if it appears inside a `begin` block
> - The cache must be per-thread (already guaranteed by `thread_local!`)
> - All 22 existing tests must continue to pass
> - Run the benchmark before and after to confirm improvement:
>   ```bash
>   time cargo run --release --features vm bench.uwu
>   time cargo run --release bench.uwu
>   ```
>
> ### 6. What NOT to change
>
> - Do not change `Compiler`, `VM`, `is_compilable`, or `expand_all`
> - Do not add serialization or `.uwuo` file support
> - Do not remove DEBUG log lines yet
>
> After implementing, show: the full `cache.rs`, the updated `vm_eval` in `mod.rs`, cache hit/miss stats on a sample run, and benchmark results before and after.