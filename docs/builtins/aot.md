---
title : Ahead-of-Time Compilation
sidebar:
  order: 9
---

### `aot-compile`

Compiles a Lisp source file to a binary AOT (Ahead-of-Time) bytecode file on disk. All compilable top-level forms are macro-expanded and compiled to bytecode chunks, then serialised to the output file.

```
(aot-compile input.pi)           →  "input.aot"
(aot-compile input.pi output.aot) →  "output.aot"
```

The first argument is the input source path (string). The optional second argument is the output path; when omitted, the output file name is derived by replacing the input extension with `.aot`.

### `aot-load`

Loads a previously compiled AOT bytecode file into the compilation cache. Subsequent evaluations of expressions whose AST matches a cached entry will use the pre-compiled bytecode directly, bypassing the compiler entirely.

```
(aot-load "file.aot")  →  ()
```

### CLI equivalents

```bash
# AOT-compile a source file
pilisp --aot hello.pi              # produces hello.aot
pilisp --aot hello.pi -o out.aot   # produces out.aot

# Run with auto-loaded bytecode
pilisp hello.pi                     # auto-loads hello.aot if present
```

When a `.pi` file is evaluated, the interpreter checks whether a sibling `.aot` file (same stem, `.aot` extension) exists. If found, it is loaded into the compile cache before evaluation begins, so every compilable expression hits the cache on its first evaluation.
