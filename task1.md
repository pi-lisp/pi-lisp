## task 1 — Bytecode & Compiler

> You are an expert Rust systems programmer. I have a working tree-walking Lisp interpreter and I want to add a **bytecode compiler** as the first phase of a two-phase VM project.
>
> ## Codebase overview
>
> - `expr.rs` — defines `Expr` (the AST) and `Env = GcHandle`
> - `gc.rs` — owns all `EnvData` frames; provides `Heap`, `GcHandle` (a `Copy` usize index)
> - `eval.rs` — current tree-walking evaluator (`eval(expr, env, heap)`)
> - `macros.rs` — macro expansion and quasiquote
> - `reader.rs` — tokenizer + parser (produces `Expr`)
> - `builtins/mod.rs` — registers built-in functions as `Expr::Func(Rc<dyn Fn(...)>)`
>
> Key constraints:
> - Macro expansion happens at eval-time; the compiler must fully expand macros **before** compiling
> - `Expr::CubicalTerm` is opaque — mark these nodes as uncompilable and return an error so the caller can fall back to the tree-walker
> - No `unsafe` code
>
> ## What I want you to implement
>
> Create two files: `vm/bytecode.rs` and `vm/compiler.rs`.
>
> **`vm/bytecode.rs`**
>
> Define a `Value` type that mirrors `Expr` but is VM-friendly (all variants `Clone`), with `value_to_expr` / `expr_to_value` helpers. Then define:
>
> ```
> pub enum Op {
>     LoadConst(Value),
>     LoadVar(String),
>     StoreVar(String),
>     Jump(usize),
>     JumpIfFalse(usize),
>     Return,
>     MakeFunc { code_offset: usize, params: Vec<String> },
>     Call(usize),
>     TailCall(usize),
>     MakeList(usize),
>     Pop,
> }
>
> pub struct Chunk {
>     pub ops: Vec<Op>,
>     pub sub_chunks: Vec<Chunk>, // for lambda bodies
> }
> ```
>
> **`vm/compiler.rs`**
>
> Implement `pub struct Compiler` with:
> - `pub fn compile(expr: &Expr, heap: &Heap) -> Result<Chunk, String>`
>
> Handle these forms:
> - literals → `LoadConst`
> - symbols → `LoadVar`
> - `quote`, `quasiquote`
> - `if` with forward-jump patching
> - `define`, `let`, `let*`, `set!`, `begin`
> - `lambda` → compile body into a sub-chunk, emit `MakeFunc`
> - function calls → emit `Call`; detect tail position and emit `TailCall` instead
> - anything containing `CubicalTerm` → return `Err("uncompilable")` immediately
>
> After each file, briefly explain any non-obvious design decisions.
> Do **not** implement the VM execution engine yet — that is phase 2.