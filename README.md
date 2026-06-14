# Cubical Lisp Interpreter

A lightweight, experimental Lisp interpreter written in Rust. Beyond standard Lisp features (macros, lexical scoping, and arithmetic), this project features a unique **cubical type theory flavor**, including **Interval types, Path applications, Dependent Function types ($\Pi$-types), and Dependent Pair types ($\Sigma$-types)**.

The interpreter operates via a two-stage process: compiling the "surface" AST (with named variables) into a bytecode-like "core" AST utilizing **De Bruijn indices** for local variable lookup, followed by evaluation.

---

## Features

### 🚀 Standard Lisp Engine

* **Lexical Scoping**: Implemented via explicit lexical environment links (`LexEnv`) with $O(1)$ fast De Bruijn variable lookup.
* **Macro System**: Built-in support for standard macros (`defmacro`) operating directly on un-evaluated S-expressions, complete with full `quasiquote`, `unquote`, and `unquote-splicing` support.
* **Core Primitives**: List manipulation (`list`, `car`, `cdr`, `cons`, `null?`), conditional branching (`if`), sequencing (`begin`), local bindings (`let`), and definitions (`define`).

### 📐 Homotopy & Cubical Primitives

* **Interval Endpoints**: Predefined endpoint constants `i0` ($0.0$) and `i1` ($1.0$).
* **Paths**: Paths represent continuous mapping lines from an interval into a type space. Use `(path (i) body)` to construct one, and `(papply path coordinate)` to evaluate an interpolation point.
* **Reflexivity**: The `refl` built-in generates a degenerate/constant path mapping any element to itself.

### 🧬 Dependent Types

* **$\Pi$-Types (Dependent Functions)**: Formulate types like `(pi (x) dom cod)`. Use `(piapply pi-type value)` to dynamically evaluate dependent codomains.
* **$\Sigma$-Types (Dependent Pairs)**: Pair constructs where the type of the second element depends on the value of the first, represented via `(sigma (x) dom cod)`. Extract or inspect structural dependencies with `(sigmacod sigma-type value)`.

---

## Architectural Layout

```
src/
├── main.rs          # Runner testing script & sample suite execution
├── expr.rs          # Core AST enum `Expr`, LexEnv variants, type implementations
├── compiler.rs      # Surface AST compilation into De Bruijn Core indices
├── eval.rs          # Main evaluation loops, function/path applications
├── builtins.rs      # Global namespace injector for arithmetic, lists, and types
├── macros.rs        # Macro substitution macro-expander and quasiquoting
├── reader.rs        # S-expression tokenizer and parser 
└── env.rs           # Re-exports global mutable environment handlers

```

---

## S-Expression Code Examples

Below is a walkthrough of features supported and executed within the interpreter environment:

### Functions & Control Flow

```lisp
;; Higher-order factorial
(define fact (lambda (n) (if (< n 1) 1 (* n (fact (- n 1))))))
(fact 10) ; => 3628800

;; Local bindings
(let ((a 3) (b 4)) (+ (* a a) (* b b))) ; => 25

```

### Advanced Macros & Quasiquoting

```lisp
;; Unless control macro
(defmacro unless (cond then) (list 'if (list 'not cond) then 0))
(unless 0 (+ 1 2)) ; => 3

;; Splicing evaluation
(define lst (list 1 2 3))
(quasiquote (start (unquote-splicing lst) end)) ; => (start 1 2 3 end)

```

### Cubical Path Interpolation

```lisp
;; Path that linearly interpolates between 1 and 5
(define interp (path (i) (+ (* (- 1 i) 1) (* i 5))))

(papply interp i0)   ; => 1.0
(papply interp i1)   ; => 5.0
(papply interp 0.5)  ; => 3.0

;; Constant reflexivity path
(define rp (refl 42))
(papply rp 0.3)      ; => 42.0

```

### Type Checking & Dependent Modifiers

```lisp
;; Dependent Pi type substitute for vectors: Vec(n) = n * n
(define vec-type (pi (n) 0 (* n n)))

(piapply vec-type 3) ; => 9.0 (instantiated codomain at n=3)
(piapply vec-type 5) ; => 25.0

;; Composing paths changing structural types dynamically over an interval
(define type-path (path (i) (pi (x) 0 (* x (+ i 1)))))
(piapply (papply type-path i0) 4) ; => 4.0  (i=0 -> cod = 4 * 1)
(piapply (papply type-path i1) 4) ; => 8.0  (i=1 -> cod = 4 * 2)

```

---

## Quick Start

Make sure you have [Rust and Cargo installed](https://www.rust-lang.org/tools/install). Clone the repository and execute the test harness containing the sample expressions:

```bash
cargo run

```