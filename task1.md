> You are an expert Rust systems programmer. The VM now correctly handles arithmetic, comparisons, `if`, `quote`, `list`, `begin`, `define`, `let`, and `let*`. The next step is to compile `lambda` expressions and closure calls entirely within the VM, without falling back to the tree-walker.
>
> ## Current state
>
> - `Op::MakeFunc { code_offset: usize, params: Vec<String> }` already exists in `bytecode.rs`
> - `Value::Closure { params, body_chunk, body_expr, env }` already exists in `bytecode.rs`
> - `is_compilable` currently returns `false` for any expression containing `lambda`
> - When `Call` resolves to a `Value::Closure`, the VM already pushes a new `CallFrame` — but this path was only reached via lambdas created by the tree-walker and converted to `Value::Closure`; now it must work for VM-compiled closures too
> - `Op::PushEnv` / `Op::PopEnv` already exist for scoping
>
> ## What I want you to implement
>
> ### 1. `vm/compiler.rs` — compile `lambda`
>
> Add a `compile_lambda` branch:
> ```
> (lambda (x y) body)
> →
>   Op::MakeFunc { sub_chunk_index: N, params: ["x", "y"] }
> ```
> The body must be compiled into a **sub-chunk** stored in `Chunk::sub_chunks[N]`, not inlined into the current chunk. The sub-chunk must end with `Op::Return`.
>
> For the body compilation, the compiler must:
> - create a new child `Compiler` context
> - add the parameter names as locally known symbols (so `LoadVar` for params compiles correctly)
> - compile the body expression into the sub-chunk
> - append `Op::Return` at the end
> - push the sub-chunk into the parent chunk's `sub_chunks` and record its index
>
> **Recursive lambdas**: if the lambda is the RHS of a `(define name (lambda ...))`, the name must be bound in the closure's env before the body executes so recursive calls work. Emit a `StoreVar(name)` of the closure itself at the start of the sub-chunk body:
> ```
> (define factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))
> →
>   sub-chunk for factorial:
>     Op::StoreSelf("factorial")   ← new op, stores the closure itself under its own name
>     [compile body]
>     Op::Return
> ```
> Add `Op::StoreSelf(String)` to `bytecode.rs` and implement it in `machine.rs` as: bind the current frame's closure value to the given name in the current env.
>
> ### 2. `vm/machine.rs` — `MakeFunc` dispatch
>
> When `Op::MakeFunc { sub_chunk_index, params }` executes:
> - retrieve `self.current_chunk().sub_chunks[sub_chunk_index].clone()`
> - capture the **current frame's env** as the closure env
> - push `Value::Closure { params, body_chunk, body_expr: Box::new(Expr::List(vec![])), env }` onto the stack
>
> When `Op::Call(argc)` resolves to `Value::Closure { params, body_chunk, env, .. }`:
> - pop `argc` args from the stack
> - allocate a new child env frame: `new_env(heap, Some(env))`
> - bind each param name to its argument value via `env_set`
> - push a new `CallFrame { chunk: body_chunk, ip: 0, stack_base: current stack top, env: child_env }`
>
> When `Op::Return` executes:
> - pop the return value from the top of the stack
> - pop the current `CallFrame`
> - push the return value onto the (now restored) caller's stack
> - if no frames remain, return the value as the final result
>
> ### 3. `vm/compiler.rs` — update `is_compilable`
>
> Remove `lambda` from the blocklist.
>
> Add a new rule: if a `lambda` body contains `CubicalTerm` or `set!` or `letrec` anywhere in its subtree, keep returning `false` for the whole expression. Use the existing deep-walk logic from `is_compilable` for this check.
>
> ### 4. `vm/compiler.rs` — compile `(define name (lambda ...))`
>
> Detect this pattern specifically and pass `name` into `compile_lambda` so it can emit `Op::StoreSelf(name)` inside the sub-chunk. For all other `define` forms the behaviour is unchanged.
>
> ### 5. Correctness requirements
>
> - Direct recursion (`factorial`) must work
> - Mutual recursion (`is-even?` / `is-odd?`) must work — for mutual recursion, `StoreSelf` is not enough; both names must already be in the global env when either body runs. Since `define` writes to the global env immediately, mutual recursion works as long as the call happens after both `define`s are evaluated, which is already the case.
> - Higher-order functions (passing lambdas as arguments, returning lambdas) must work
> - Closures must capture variables by reference to the env frame, not by value — this is already guaranteed since `GcHandle` is used
> - `(define f (lambda (x) (lambda (y) (+ x y))))` (currying) must work
> - All previously passing tests must continue to pass
>
> ### 6. What NOT to change
>
> - Do not implement `quasiquote` compilation yet
> - Do not implement `set!` compilation yet
> - Do not remove DEBUG log lines yet
> - Do not change the `CubicalTerm` fallback path
>
> After implementing, show: the updated `is_compilable`, `compile_lambda`, `MakeFunc` and `StoreSelf` dispatch in the run loop, and confirm all existing tests still pass.