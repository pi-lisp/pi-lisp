---
title : Comparisons
sidebar:
  order: 4
---

All numeric comparisons take exactly two number arguments and return `#t` for true, `#f` for false.

| Function | Description         |
|----------|---------------------|
| `=`      | Equal               |
| `<`      | Less than           |
| `>`      | Greater than        |
| `<=`     | Less than or equal  |
| `>=`     | Greater than or equal |

```
(= a b)   →  #t or #f
(< a b)   →  #t or #f
```

`=` works with complex numbers — it compares both the real and imaginary parts. `<`, `>`, `<=`, `>=` raise an error when given a complex number with a non-zero imaginary part (complex numbers have no total ordering).

```lisp
(= 1+2i 1+2i)   ; ⇒ #t
(= 1+2i 3+4i)   ; ⇒ #f
(< 1+2i 3+4i)   ; error: expected real number, got complex 1+2i
```

---

### `not`
Logical negation. Returns `#t` if the argument is falsy, `#f` otherwise.

```
(not x)  →  #t or #f
```

Expects exactly one argument.
