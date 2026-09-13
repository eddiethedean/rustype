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

## Open decisions

The following still require prototyping or specification work:

- whether plain Python `def` is accepted in `.rpy`
- exact empty `Option` variant spelling and syntax
- runtime representation of newtypes
- runtime representation of enums/variants
- exact semantics for `Option` with `?`
- Python exception-to-`Result` adapter conventions
- how Python `Any`/unknown values appear in the Rustype type lattice
- authoritative parser implementation strategy
- source-map storage and traceback integration format
- packaging/build-backend strategy
- exact minimum supported Python version
