# Rust Capability Matrix

Rustype does not copy Rust wholesale. This matrix classifies major Rust concepts as **Adopt**, **Adapt**, **Defer**, or **Reject** based on whether they improve Python application correctness without requiring Rust's memory model.

| Rust concept | Rustype decision | Rustype direction |
|---|---|---|
| `Result<T, E>` | Adopt | First-class recoverable error type |
| `Option<T>` | Adopt | First-class optional-value type |
| `Ok` / `Err` | Adopt | Result variants |
| `Some` / `None` | Adapt | Option variants with lowering compatible with Python |
| `?` operator | Adopt | Static propagation of compatible result values |
| data-carrying enums | Adopt | Closed algebraic sum types |
| exhaustive `match` | Adopt | Compile-time exhaustiveness for closed types |
| `if let` | Adopt | Concise pattern narrowing |
| traits | Adopt | Behavioral interfaces, preferably structural |
| trait bounds | Adopt | Generic capability constraints |
| newtype pattern | Adopt | First-class `newtype` declaration |
| `Never` / `!` | Adapt | Use Rustype/Python-compatible `Never` semantics |
| panic | Adopt | Explicit unrecoverable invariant failure |
| unsafe blocks | Adapt | Static-safety escape hatch, never memory unsafe |
| safe wrappers around unsafe | Adopt | Core design pattern for Python interop |
| immutable bindings | Defer | Useful, but Python mutation semantics require careful design |
| `let` / `let mut` | Defer | Consider only if immutability guarantees justify syntax |
| typestate patterns | Adopt | Implement through generics/marker states first |
| pattern destructuring | Adopt/Adapt | Build on Python-like matching |
| iterators/combinators | Adapt | Preserve Python iterators; add Result/Option combinators |
| associated types | Defer | Only if real Python use cases justify complexity |
| associated constants | Defer | Evaluate with traits later |
| default trait methods | Defer | Possible after core trait semantics stabilize |
| `impl Trait` | Defer | Explore Python-compatible opaque/interface return forms |
| `dyn Trait` | Defer | Python is already runtime-dynamic; must justify distinct semantics |
| `where` clauses | Defer | Add only when inline bounds become inadequate |
| automatic error conversion (`From`) | Adapt/Defer | Explicit conversion model first |
| macros | Reject initially | Python metaprogramming and decorators already exist; compiler macros are a separate language project |
| procedural macros | Reject initially | High complexity and poor fit for early goals |
| ownership | Reject | Explicitly outside project scope |
| borrowing | Reject | Explicitly outside project scope |
| borrow checker | Reject | Explicitly outside project scope |
| lifetimes | Reject | Explicitly outside project scope |
| move semantics | Reject | No Rust memory ownership model |
| `Copy` memory semantics | Reject | No Rust ownership model |
| `Drop` memory semantics | Reject | Python runtime owns object lifetime |
| raw pointers | Reject | Not a Rustype language concern |
| `Box<T>` | Reject | Memory allocation abstraction |
| `Rc<T>` / `Arc<T>` | Reject | Ownership/memory abstractions |
| allocator APIs | Reject | CPython/runtime responsibility |
| layout guarantees | Reject | Python object model governs representation |
| `Send` / `Sync` memory guarantees | Reject | Rust memory/thread-safety semantics do not transfer directly |
| const evaluation | Defer | Consider only for concrete compile-time modeling needs |
| const generics | Defer | Consider only when Python application use cases justify them |
| specialization | Reject/defer | Too much complexity for initial trait model |
| GATs/HKTs | Reject/defer | Not required for initial product goals |

## Decision rule

A Rust concept belongs in Rustype only when it satisfies all of these tests:

1. It solves a meaningful Python correctness or API-modeling problem.
2. It can be explained without Rust ownership/borrowing knowledge.
3. It can lower predictably to CPython semantics.
4. It provides enough value to justify any new syntax or runtime support.
5. It does not unnecessarily isolate Rustype from Python libraries.

This matrix should be updated as design decisions become stable.
