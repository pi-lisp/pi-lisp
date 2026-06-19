---
title : Cubical Type Theory
sidebar:
  order: 8
---

These builtins provide a surface-level Lisp API over the internal cubical type theory (CTT) kernel. Every cubical builtin returns an `Expr::CubicalTerm` wrapping the corresponding `Term` variant. Arguments that are expected to be cubical terms must themselves be `Expr::CubicalTerm` values.

### `Interval Atoms`

#### `interval-zero`
The interval endpoint `0`.

```
(interval-zero)  →  TInterval(I0)
```

No arguments.

---

#### `interval-one`
The interval endpoint `1`.

```
(interval-one)  →  TInterval(I1)
```

No arguments.

---

#### `interval-var`
An interval variable identified by a numeric index.

```
(interval-var n)  →  TInterval(IVar(n))
```

`n` is cast to `i32`.

---

#### `interval-meet`
The meet (minimum / conjunction) of two interval expressions, normalised to DNF immediately.

```
(interval-meet a b)  →  TCube(dnf)
```

Both arguments must be `TInterval` terms (i.e. produced by `interval-var`, `interval-zero`, or `interval-one`). Passing a pre-normalised `TCube` is an error.

---

#### `interval-join`
The join (maximum / disjunction) of two interval expressions, normalised to DNF immediately.

```
(interval-join a b)  →  TCube(dnf)
```

Both arguments must be `TInterval` terms. Passing a pre-normalised `TCube` is an error.

---

#### `interval-neg`
The negation of an interval expression, normalised to DNF immediately.

```
(interval-neg a)  →  TCube(dnf)
```

Argument must be a `TInterval` term. Passing a pre-normalised `TCube` is an error.

---

### `Variables and Universes`

#### `var`
A de Bruijn-indexed term variable.

```
(var n)  →  TVar(n)
```

`n` is cast to `i32`.

---

#### `univ`
A universe at the given level.

```
(univ level)  →  TUniv(level)
```

`level` is cast to `i32`.

---

#### `interval-type`
The interval type constant `𝕀` itself (not a term of the interval).

```
(interval-type)  →  TIntervalTy
```

No arguments.

---

### `Functions`

#### `clambda`
A term-level lambda abstraction. Named `clambda` (rather than `lambda`) to avoid shadowing the Lisp special form.

```
(clambda name body)  →  TAbs(name, body)
```

`name` must be a symbol; `body` must be a cubical term.

---

#### `app`
Function application.

```
(app f x)  →  TApp(f, x)
```

Both arguments must be cubical terms.

---

#### `pi`
A dependent function type (Π-type).

```
(pi name domain codomain)  →  TPi(name, domain, codomain)
```

`name` must be a symbol; `domain` and `codomain` must be cubical terms.

---

### `Path Types`

#### `path-type`
The identity/path type between two terms over a given type.

```
(path-type A a b)  →  TPath(A, a, b)
```

All three arguments must be cubical terms.

---

#### `path-lambda`
A path abstraction (binder over the interval).

```
(path-lambda name body)  →  PLam(name, body)
```

`name` must be a symbol; `body` must be a cubical term.

---

#### `path-app`
Applies a path to an interval point.

```
(path-app p i)  →  PApp(p, i)
```

Both arguments must be cubical terms.

---

### `Sigma Types and Pairs`

#### `sigma`
A dependent pair type (Σ-type).

```
(sigma name domain codomain)  →  TSigma(name, domain, codomain)
```

`name` must be a symbol; `domain` and `codomain` must be cubical terms.

---

#### `pair`
A dependent pair value.

```
(pair a b)  →  TPair(a, b)
```

Both arguments must be cubical terms.

---

#### `fst`
Projects the first component of a pair.

```
(fst p)  →  TFst(p)
```

`p` must be a cubical term.

---

#### `snd`
Projects the second component of a pair.

```
(snd p)  →  TSnd(p)
```

`p` must be a cubical term.

---

### `Composition and Transport`

#### `hcomp`
Homogeneous composition. Fills a cube with a given face constraint.

```
(hcomp A phi tube base)  →  THComp(A, phi, tube, base)
```

| Argument | Role |
|----------|------|
| `A`      | The type |
| `phi`    | Face formula |
| `tube`   | The partial element (the tube) |
| `base`   | The base element at `i=0` |

All four arguments must be cubical terms.

---

#### `transport`
Transports a term along a path of types.

```
(transport path x)  →  TTransport(path, x)
```

Both arguments must be cubical terms. `path` is a path in a universe; `x` is the element to transport.

---

### `Equivalences and Univalence`

#### `equiv`
The type of equivalences between two types.

```
(equiv A B)  →  TEquiv(A, B)
```

Both arguments must be cubical terms.

---

#### `make-equiv`
Constructs an equivalence from its components.

```
(make-equiv A B f g eta eps)  →  TMkEquiv(A, B, f, g, eta, eps)
```

| Argument | Role |
|----------|------|
| `A`      | Source type |
| `B`      | Target type |
| `f`      | Forward map `A → B` |
| `g`      | Backward map `B → A` |
| `eta`    | Left inverse homotopy |
| `eps`    | Right inverse homotopy |

All six arguments must be cubical terms.

---

#### `equiv-fwd`
Applies the forward direction of an equivalence to a term.

```
(equiv-fwd e x)  →  TEquivFwd(e, x)
```

Both arguments must be cubical terms.

---

#### `ua`
Univalence: converts an equivalence into a path between types.

```
(ua e)  →  TUa(e)
```

`e` must be a cubical term representing an equivalence.

---

### `Glue Types`

Glue types are used to implement the computational content of univalence.

#### `glue`
Forms a Glue type. The third argument `T` must bundle the equivalent-type family and the equivalence together as a `pair` term.

```
(glue A phi T)  →  TGlue(A, phi, T)
```

`T` should be constructed as `(pair type equiv)`. All three arguments must be cubical terms.

---

#### `glue-elem`
Constructs a term of a Glue type.

```
(glue-elem phi t a)  →  TGlueElem(phi, t, a)
```

| Argument | Role |
|----------|------|
| `phi`    | Face formula |
| `t`      | Element on the glued side |
| `a`      | Underlying element in the base type |

All three arguments must be cubical terms.

---

#### `unglue`
Extracts the underlying base-type element from a glued term.

```
(unglue phi te g)  →  TUnglue(phi, te, g)
```

| Argument | Role |
|----------|------|
| `phi`    | Face formula |
| `te`     | Bundled `(type, equiv)` pair |
| `g`      | The glued term to unglue |

All three arguments must be cubical terms.

---

### `Inductive and Higher-Inductive Types`

These builtins construct terms of declared inductive and higher-inductive types (HITs). The datatype schema (constructors, path-constructor boundaries) must be registered in the environment via the `Env` integration before type-checking terms built with these forms; the constructors themselves can be built and evaluated without a schema.

#### `data-type`
The type of a declared inductive or higher-inductive datatype, as a cubical term.

```
(data-type name)  →  TData(name)
```

`name` must be a symbol matching a declared datatype.

---

#### `con`
An ordinary constructor application.

```
(con datatype constructor arg0 arg1 ...)  →  TCon(datatype, constructor, args)
```

| Argument      | Role |
|---------------|------|
| `datatype`    | Symbol — the datatype this constructor belongs to |
| `constructor` | Symbol — the constructor name |
| `arg0 ...`    | Zero or more cubical term arguments |

The datatype name is stored alongside the constructor name so that evaluation and type-checking do not require a global name lookup that could collide across datatypes.

---

#### `pcon`
A path-constructor application. Path constructors take an interval argument in addition to any ordinary arguments.

```
(pcon datatype pconstructor r arg0 arg1 ...)  →  TPCon(datatype, pconstructor, args, r)
```

| Argument       | Role |
|----------------|------|
| `datatype`     | Symbol — the datatype |
| `pconstructor` | Symbol — the path constructor name |
| `r`            | Cubical term — the interval argument |
| `arg0 ...`     | Zero or more ordinary cubical term arguments |

The interval argument `r` is listed third (immediately after the constructor name) to match the surface reading "path constructor applied to an interval point". Internally it is stored as the last field of `TPCon`.

---

#### `elim`
The eliminator (recursor) for an inductive or higher-inductive type.

```
(elim motive scrutinee case0 case1 ...)  →  TElim(motive, cases, scrutinee)
```

| Argument     | Role |
|--------------|------|
| `motive`     | Cubical term — the return-type motive (a function over the datatype) |
| `scrutinee`  | Cubical term — the element being eliminated |
| `caseN`      | A case clause (see below) |

Each case clause is a list of the form:

```
(constructor-name binder0 binder1 ... body)
```

The first element is a symbol naming the constructor this case handles. Everything between the constructor name and the final element is a binder name (symbol). The final element is the case body, a cubical term. For a path-constructor case the interval binder is listed last among the binders, and the body is expected to be a `PLam`-shaped term over that variable.

**Example — S¹ eliminator:**

```lisp
(elim
  motive                        ; the motive P : S¹ → U
  scrutinee                     ; the element of S¹ to eliminate
  (base body-base)              ; ordinary constructor case: no binders
  (loop i body-loop)            ; path constructor case: interval binder i
)
```

`body-loop` must be a `(path-lambda ...)` term whose application at `(interval-zero)` and `(interval-one)` agrees with `body-base`, matching the path-constructor boundary equations declared in the datatype schema.

---

### `Evaluation and Type-Checking`

#### `ctt-eval`
Normalises a closed cubical term.

```
(ctt-eval t)  →  CubicalTerm
```

Returns the normal form of `t`.

---

#### `ctt-infer`
Infers the type of a closed cubical term.

```
(ctt-infer t)  →  CubicalTerm
```

Returns the inferred type as a cubical term. Errors if type inference fails. For terms involving `data-type`, `con`, `pcon`, or `elim`, the relevant datatype schemas must be registered in the environment; this builtin passes an empty schema slice and will error on unregistered datatypes.

---

#### `ctt-check`
Checks that a term has a given type in the empty context.

```
(ctt-check t ty)  →  1.0
```

Returns `1.0` on success. Raises a Lisp error on type-checking failure. For terms involving inductive types, the relevant datatype schemas must be registered in the environment; this builtin passes an empty schema slice and will error on unregistered datatypes.

---

#### `ctt-equal?`
Tests definitional equality of two closed cubical terms.

```
(ctt-equal? t u)  →  1.0 or 0.0
```

Returns `1.0` if `t` and `u` are definitionally equal, `0.0` otherwise.
