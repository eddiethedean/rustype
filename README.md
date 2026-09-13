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

## Early language sketch

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

The generated Python should remain inspectable, debuggable, interoperable with normal `.py` modules, and compatible with the Python package ecosystem.

## Core goals

- Preserve CPython as the runtime.
- Preserve normal Python imports and PyPI compatibility.
- Implement the compiler and semantic engine in Rust.
- Make recoverable failure explicit with `Result[T, E]`.
- Make optionality explicit with `Option[T]`.
- Support data-carrying enums and exhaustive matching.
- Model interfaces as traits backed by Python-compatible abstractions.
- Support newtypes and typestate-oriented APIs.
- Make dynamic or weakly typed behavior explicit through `unsafe` boundaries.
- Produce source-aware diagnostics and tracebacks that point to `.rpy`, not generated Python.
- Provide first-class editor tooling through a shared compiler/LSP core.

## Explicit non-goals

Rustype will not initially attempt to provide:

- ownership or borrow checking
- lifetimes
- native-code compilation
- a new garbage collector or object model
- a new package repository
- a replacement for CPython
- performance-oriented Rust abstractions such as `Box`, `Rc`, or `Arc`
- arbitrary Rust syntax for its own sake

## Planned documentation

The `docs/` directory contains the working design specification, compiler architecture, interoperability plan, safety model, roadmap, and implementation phases.

## Status

Rustype is in the language-design and architecture phase. Syntax and semantics in these documents are proposals until the specification is stabilized.
