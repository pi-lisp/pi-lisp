---
title : Arithmetic
sidebar:
  order: 3
---

All arithmetic functions transparently support **complex numbers**. If any argument is a complex number (literal like `1+2i` or produced by `make-rectangular`), all arguments are promoted to complex and the result is a complex number.

### `+`
Sums zero or more numbers.

```
(+ n1 n2 ...)  →  Number
```

Returns `0` when called with no arguments.

Examples:
```lisp
(+ 1 2 3)       ; ⇒ 6
(+ 1+2i 3+4i)  ; ⇒ 4+6i
(+ 5 1+2i)      ; ⇒ 6+2i
```

---

### `-`
Subtracts numbers. With a single argument, negates it.

```
(- n)           →  Number   ; negation
(- n1 n2 ...)   →  Number   ; subtraction
```

Requires at least one argument. Negation of `i64::MIN` (`-9223372036854775808`) raises an overflow error.

Examples:
```lisp
(- 10 3)        ; ⇒ 7
(- 1+2i)        ; ⇒ -1-2i
(- 1+2i 3+4i)  ; ⇒ -2-2i
```

---

### `*`
Multiplies zero or more numbers.

```
(* n1 n2 ...)  →  Number
```

Returns `1` when called with no arguments.

Examples:
```lisp
(* 2 3 4)       ; ⇒ 24
(* 2i 3i)       ; ⇒ -6
(* 1+2i 3+4i)  ; ⇒ -5+10i
```

---

### `/`
Divides the first number by each subsequent number.

```
(/ n1 n2 ...)  →  Number
```

Requires at least one argument. Raises an error on division by zero.

Examples:
```lisp
(/ 10 2)        ; ⇒ 5
(/ 1+2i 1+1i)  ; ⇒ 1.5+0.5i
```

### `%`
Returns the remainder of dividing the first number by the second.

```
(% n1 n2)  →  Number
```

Requires exactly two arguments. Does not support complex numbers.

---

### Complex-specific builtins

| Function | Signature | Description |
|----------|-----------|-------------|
| `real-part` | `(real-part z) → Float` | Real part of a complex number |
| `imag-part` | `(imag-part z) → Float` | Imaginary part of a complex number |
| `magnitude` | `(magnitude z) → Float` | Magnitude (modulus) of a complex number |
| `angle` | `(angle z) → Float` | Phase angle of a complex number (atan2) |
| `make-rectangular` | `(make-rectangular re im) → Complex` | Constructs a complex number from real and imaginary parts |
| `make-polar` | `(make-polar r theta) → Complex` | Constructs a complex number from polar coordinates |

```lisp
(real-part 3+4i)        ; ⇒ 3
(imag-part 3+4i)        ; ⇒ 4
(magnitude 3+4i)        ; ⇒ 5
(angle 1+1i)            ; ⇒ 0.7853981633974483  (= pi/4)
(make-rectangular 2 3)  ; ⇒ 2+3i
(make-polar 1 0)        ; ⇒ 1+0i
```
