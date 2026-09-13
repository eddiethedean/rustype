# Rustype

**Rust-inspired type safety for Python.**

Rustype is a planned `.rpy` language and compiler that brings Rust's type-driven correctness concepts to Python while preserving Python's runtime and ecosystem.

Rustype source compiles to ordinary Python:

```text
app.rpy -> rustype compiler (Rust) -> app.py -> CPython
```

Rustype deliberately adopts Rust concepts such as `Result`, `Option`, enums with data-carrying variants, traits, newtypes, exhaustive matching, typestate, explicit `unsafe` regions, and `?`-style error propagation.

Rustype deliberately does **not** implement Rust's memory model. Ownership, borrowing, lifetimes, move semantics, pointer aliasing, allocation rules, and borrow checking are out of scope.

## Design principle

> Borrow Rust's correctness concepts and guarantees when they improve Python; keep Python's runtime, object model, library ecosystem, and ergonomics.

Rustype is not Rust-with-Python-syntax and not Python-with-random-Rust-keywords. Rust-inspired syntax is introduced only when it expresses a useful static guarantee more clearly than Python can.

## v0 language sketch

```python
from fastapi import FastAPI

app = FastAPI()

newtype UserId = int

enum UserError:
    NotFound(id: UserId)
    Database(message: str)

trait UserRepository:
    fn find(self, id: UserId) -> Result[User, UserError]

@app.get("/users/{user_id}")
async fn get_user(user_id: UserId) -> Result[User, UserError]:
    user = repository.find(user_id)?
    return Ok(user)
```

Rustype v0 uses `fn`/`async fn`, closed algebraic enums, structural traits, first-class `Option`/`Result`, postfix `?`, exhaustive `match`, `if let`, `newtype`, and explicit `unsafe` blocks. Plain Python `def` is intentionally not accepted in `.rpy` v0 source.

The generated Python should remain inspectable, debuggable, interoperable with normal `.py` modules, and compatible with the Python package ecosystem.

## Core goals

- Preserve CPython as the runtime.
- Preserve normal Python imports and PyPI compatibility.
- Implement the compiler and semantic engine in Rust.
- Make recoverable failure explicit with `Result[T, E]`.
- Make optionality explicit with `Option[T]`.
- Support data-carrying enums and exhaustive matching.
- Model interfaces as traits backed by Python-compatible structural abstractions.
- Support newtypes and typestate-oriented APIs.
- Make dynamic or weakly typed behavior explicit through `unsafe` boundaries.
- Produce source-aware diagnostics and tracebacks that point to `.rpy`, not generated Python.
- Provide first-class editor tooling through a shared compiler/LSP core.

## Explicit non-goals

Rustype does not attempt to provide:

- ownership or borrow checking
- lifetimes
- native-code compilation
- a new garbage collector or object model
- a new package repository
- a replacement for CPython
- Rust memory abstractions such as `Box`, `Rc`, or `Arc`
- arbitrary Rust syntax for its own sake

## Compiler bootstrap

The repository now contains the first executable Rust compiler skeleton:

```text
Cargo.toml
crates/
  rustype-ast/
  rustype-parser/
  rustype-cli/
runtime/
  rustype_runtime/
tests/
  fixtures/
```

The bootstrap parser currently recognizes the first vertical slice: source-spanned `newtype` declarations and typed `fn`/`async fn` headers. It also emits stable diagnostic codes for unsupported or invalid bootstrap syntax.

Run the current checker with:

```bash
cargo run -p rustype-cli -- check tests/fixtures/pass/first_slice.rpy
```

The bootstrap intentionally uses no third-party Rust dependencies. The authoritative parser strategy will be selected only after the core AST and grammar have been exercised by real implementation work.

## Language specification

The first concrete language contract is documented in:

- `docs/V0_SYNTAX_SPEC.md` — normative v0 syntax and feature rules
- `docs/GRAMMAR.md` — EBNF-like parser grammar
- `docs/AST_MODEL.md` — parser AST and source-span contract
- `docs/HIR_MODEL.md` — resolved semantic representation
- `docs/LOWERING_IR.md` — explicit lowering representation
- `docs/TYPE_SYSTEM.md` — static type model
- `docs/ERROR_MODEL.md` — `Result`, `Option`, propagation, exceptions, and panic
- `docs/SAFETY_MODEL.md` — safe vs `unsafe` static boundaries
- `docs/DECISIONS.md` — accepted architecture and syntax decisions

The rest of `docs/` contains compiler architecture, Python interoperability, tooling, roadmap, and implementation planning.

## Status

Rustype has moved from design-only into compiler prototyping. The v0 syntax, AST/HIR/lowering contracts, Rust workspace, source-span types, bootstrap parser, CLI checker, runtime ABI scaffold, fixtures, and CI gates are now in the repository. The next implementation slice is to replace the bootstrap line parser with the authoritative lexer/parser and begin Python code generation for `newtype` plus `fn`.