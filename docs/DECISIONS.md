# Design Decisions

This file records high-level decisions that should remain stable unless deliberately revisited.

## D001 — Rustype uses `.rpy`

**Status:** Accepted

Rustype is a distinct source language using `.rpy` files rather than a lint/profile layered entirely on `.py`.

Reasoning:

- allows syntax such as `fn`, `trait`, `enum`, `newtype`, postfix `?`, and explicit `unsafe`
- lets Rustype define coherent semantics rather than stretching Python syntax with runtime helpers
- preserves Python as the compilation target

## D002 — Rustype compiles to Python

**Status:** Accepted

Rustype lowers to ordinary Python and runs on CPython.

Rustype does not introduce a new VM, garbage collector, or execution runtime.

## D003 — Compiler implementation is Rust

**Status:** Accepted

The compiler, semantic engine, CLI, and eventual LSP core should be implemented in Rust.

The generated program remains Python.

## D004 — Python ecosystem compatibility is foundational

**Status:** Accepted

Rustype uses Python's package ecosystem rather than creating a parallel registry.

Normal Python imports, typed libraries, frameworks, and packaging should work with the least possible friction.

## D005 — Memory management is explicitly out of scope

**Status:** Accepted

Rustype will not implement Rust ownership, borrowing, lifetimes, borrow checking, pointer aliasing rules, move-based memory semantics, or allocator control.

This is a permanent scope boundary under the current project mission, not merely a post-1.0 backlog item.

## D006 — Rustype adopts Rust correctness concepts selectively

**Status:** Accepted

Rustype borrows Rust concepts when they provide meaningful type-safety, state-modeling, or error-handling improvements for Python applications.

Rust syntax is not a goal by itself.

## D007 — `Result` and `Option` are first-class

**Status:** Accepted

Rustype will use first-class `Result[T, E]` and `Option[T]` semantics rather than relying solely on Python exceptions and `T | None`.

## D008 — Postfix `?` is a target language feature

**Status:** Accepted

Rustype will support Rust-inspired propagation syntax for compatible result-like values, subject to precise semantic rules.

## D009 — Rustype enums are closed algebraic types

**Status:** Accepted

Enums may contain data-carrying variants and enable compiler-enforced exhaustive matching.

They are not merely aliases for Python's `enum.Enum` behavior.

## D010 — Traits are behavioral contracts

**Status:** Accepted

Rustype traits should map naturally onto Python's structural typing model where possible rather than imposing Rust-style nominal implementation rules without need.

## D011 — `unsafe` means static-safety escape, not memory unsafe

**Status:** Accepted

Rustype `unsafe` regions explicitly mark operations the compiler cannot prove under normal Rustype guarantees, especially dynamic Python interop.

The term never grants raw-memory capabilities.

## D012 — Rustype owns Rustype semantics

**Status:** Accepted

Pyright, BasedPyright, ty, or other Python type checkers may verify generated Python, but they do not define the semantics of `.rpy`.

## D013 — Source mapping is architectural, not polish

**Status:** Accepted

Every lowering stage must preserve source origin so diagnostics and tracebacks can refer to `.rpy` locations. Source mapping cannot be postponed until after the compiler is built.

## D014 — Generated Python should remain inspectable

**Status:** Accepted

Generated code does not need to be hand-written-quality Python, but it should be deterministic, valid, readable enough to diagnose, and friendly to Python tooling.

## D015 — Language features require semantic justification

**Status:** Accepted

A Rust-derived feature is admitted only when it solves a real Python correctness/modeling problem, lowers predictably to Python, and does not depend on Rust memory semantics.

## D016 — Rustype functions use `fn`, not `def`, in v0

**Status:** Accepted

Rustype-defined functions and methods use `fn` or `async fn`.

Plain Python `def` and `async def` are rejected in `.rpy` v0 source. Python functions remain available through imported `.py` modules.

This keeps the language boundary explicit and prevents Python declarations from silently receiving Rustype semantics.

## D017 — Rustype `match` arms omit Python's `case` keyword

**Status:** Accepted

Rustype uses:

```python
match message:
    Quit:
        ...
    Move(x, y):
        ...
```

rather than Python's `case` arm syntax.

This makes closed algebraic matching concise and gives the parser a clear Rustype-specific pattern grammar.

## D018 — `None` is the empty `Option` variant spelling

**Status:** Accepted for v0

Rustype uses `Some(value)` and `None` for `Option[T]`.

The compiler disambiguates the `Option` empty variant from ordinary Python `None` using type context.

## D019 — `?` supports both `Result` and `Option` in v0

**Status:** Accepted

For `Result[T, E]`, postfix `?` unwraps `Ok(T)` and propagates `Err(E)` from a compatible `Result`-returning function.

For `Option[T]`, postfix `?` unwraps `Some(T)` and propagates `None` from an `Option`-returning function.

Cross-conversion between `Option` and `Result` is not implicit in v0.

## D020 — Rustype traits are structural in v0

**Status:** Accepted

A class satisfies a trait when its public method signatures are compatible. Explicit Rust-style `impl Trait for Type` blocks are not part of v0.

This preserves Python's natural structural interface model while retaining Rust-inspired trait terminology and bounds.

## D021 — Standalone `let` and `let mut` are deferred

**Status:** Accepted

Rustype v0 retains Python assignment syntax. `let` and `mut` are reserved for future design work but are rejected as standalone declarations.

`let` is valid only as part of the v0 `if let` construct.

## D022 — v0 type expressions are deliberately constrained

**Status:** Accepted

Declared Rustype types use Python-style names, subscriptions, and unions. Arbitrary runtime expressions, calls, lambdas, or computed values are not legal type expressions.

This keeps the semantic model tractable and makes lowering predictable.

## D023 — Bare `except:` is rejected in v0

**Status:** Accepted

Rustype requires an explicit exception type in `except` clauses. This aligns with Rustype's goal of making failure boundaries explicit while preserving Python exception interoperability.

## D024 — `yield`, walrus expressions, and metaclass syntax are deferred from v0

**Status:** Accepted

The first compiler deliberately excludes generator semantics, `:=`, and metaclass declarations. These features can be revisited only after the core parser, type system, lowering, and source-map pipeline are stable.

## D025 — Compiler representations are split into AST, HIR, and lowering IR

**Status:** Accepted

Rustype uses three distinct compiler representations:

```text
AST -> HIR -> lowering IR -> Python
```

The AST represents source syntax only. HIR owns resolved names, semantic types, variants, traits, propagation modes, exhaustiveness, and safety classification. Lowering IR owns explicit control flow and backend-oriented operations.

This separation is required to keep parsing, semantics, and Python code generation independently testable.

## D026 — The parser never desugars postfix `?`

**Status:** Accepted

The parser represents postfix `?` as a first-class `TryExpr` node.

HIR determines whether it is `Result` or `Option` propagation and validates the enclosing return type. Only lowering IR expands it into early-return control flow.

This prevents syntax processing from depending on type information.

## D027 — Every compiler node preserves source origin

**Status:** Accepted

AST nodes carry source spans. HIR nodes preserve source origin. Lowering nodes and compiler-generated temporaries retain an origin pointing back to the relevant `.rpy` span.

Source positions are represented using UTF-8 byte offsets with line/column information derived from a file line index.

This is the foundation for diagnostics, source maps, and Rustype-aware tracebacks.

## D028 — Dynamic Python is represented explicitly in the semantic type model

**Status:** Accepted

Untyped or insufficiently typed Python interoperability resolves to a dedicated dynamic semantic type rather than silently behaving as universally safe `Any`.

Operations that Rustype cannot prove safe must either be narrowed/validated according to language rules or occur within an explicit `unsafe` boundary.

The internal working name for this type is `PythonDynamic`.

## D029 — Lowering IR is structured control flow, not SSA, in v0

**Status:** Accepted

The v0 lowering IR uses explicit blocks, temporaries, branches, variant dispatch, returns, and runtime operations without requiring SSA form.

Block and temporary IDs should nevertheless be stable compiler identities so a CFG or SSA layer can be added later without redesigning source semantics.

## D030 — `Result` error propagation uses identity compatibility in v0

**Status:** Accepted

For v0, postfix `?` on `Result[T, E]` propagates errors when the source error type is directly compatible with the enclosing `Result` error type.

Rust-style `From` conversion or an equivalent generalized conversion mechanism is deferred. HIR records propagation compatibility explicitly so this can be extended later without changing syntax.

## Open decisions

The following still require prototyping or later specification work:

- runtime representation of newtypes
- runtime representation of enums/variants
- Python exception-to-`Result` adapter conventions
- authoritative parser implementation library/strategy
- source-map storage and traceback integration format
- packaging/build-backend strategy
- exact minimum supported Python version
- whether `let`/`let mut` should become first-class post-v0 syntax
- whether explicit trait `impl` blocks add enough value for a later language version
- whether `where` clauses and associated types belong in Rustype
- exact shadowing rules for Rustype intrinsics such as `Some`, `Ok`, and `Err`
