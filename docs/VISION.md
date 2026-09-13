# Vision

Rustype is a Python-targeting language focused on type-driven correctness.

Its central thesis is simple:

> Python can benefit from many of Rust's type-system ideas without adopting Rust's memory-management model.

Rustype therefore separates Rust into two conceptual halves.

## What Rustype adopts

Rustype adopts concepts that improve correctness, modeling, API clarity, and failure handling:

- `Result[T, E]`
- `Option[T]`
- `Ok`, `Err`, `Some`, and the empty option variant
- data-carrying enums
- exhaustive pattern matching
- traits and trait bounds
- newtypes
- typestate
- `Never`-like uninhabited control flow
- explicit panic/unrecoverable failure
- explicit `unsafe` regions
- safe abstractions over unsafe internals
- explicit error propagation inspired by `?`
- immutability where it improves reasoning

## What Rustype rejects

Rustype intentionally does not adopt Rust features whose primary purpose is memory safety or native memory control:

- ownership
- borrowing
- lifetimes
- borrow checking
- move semantics as a memory rule
- `Copy`/`Drop` memory semantics
- pointer aliasing rules
- allocator control
- stack/heap placement guarantees
- `Box`, `Rc`, `Arc`, and related memory ownership abstractions

Rustype values are Python objects. CPython remains responsible for object lifetime and memory management.

## Product identity

Rustype should be understood as a **language frontend for Python**, not as a new runtime.

```text
Rustype syntax + Rust-inspired type model
                  |
                  v
            generated Python
                  |
                  v
                CPython
```

This allows Rustype to gain stronger syntax and semantics while inheriting Python's enormous runtime and package ecosystem.

## Compatibility philosophy

Rustype should make ordinary Python easy to call and ordinary Python libraries easy to import. The project should not require the ecosystem to be rewritten.

At the same time, `.rpy` modules may intentionally reject patterns that are valid in `.py` when those patterns undermine Rustype's guarantees.

## Syntax philosophy

Rustype should remain recognizably Pythonic. Rust syntax should be borrowed only when it expresses a static guarantee or modeling concept materially better than existing Python syntax.

Good candidates include:

- `fn`
- `enum`
- `trait`
- `newtype`
- `Result`
- `Option`
- `?`
- explicit `unsafe`
- exhaustive matching
- `if let`

Poor candidates include syntax whose meaning depends on Rust's memory model.

## Success criteria

Rustype succeeds if a Python developer can:

1. learn the language incrementally,
2. use existing Python libraries immediately,
3. model states and failures more precisely than in normal Python,
4. receive compiler-grade diagnostics before runtime,
5. inspect generated Python when needed,
6. debug against `.rpy` source rather than compiler output,
7. mix `.rpy` and `.py` in the same project,
8. deploy on ordinary Python infrastructure.

Rustype should feel like a safer way to write Python applications, not a replacement ecosystem users must migrate into.
