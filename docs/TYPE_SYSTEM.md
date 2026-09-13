# Type System

Rustype's type system is Rust-inspired, Python-targeted, and intentionally independent of Rust's ownership model.

## Goals

The type system should make invalid states and unhandled failures harder to express while remaining interoperable with Python libraries and values.

Core guarantees include:

- explicit optionality
- explicit recoverable failure
- closed sum types
- exhaustive matching
- structural behavioral contracts
- distinct semantic newtypes
- generic constraints
- typestate-friendly APIs
- explicit static-safety escape hatches

## Type categories

Rustype should initially support:

- Python scalar/runtime types such as `int`, `str`, `float`, `bool`, `bytes`
- user classes
- generics
- tuples, mappings, sequences, sets, and iterators
- `Option[T]`
- `Result[T, E]`
- Rustype enums/variants
- traits
- newtypes
- unions where interoperability requires them
- `Never`
- callable types
- async/await-related types

## `Option[T]`

`Option[T]` models the presence or absence of a value. It is conceptually a closed enum with `Some(T)` and `None` variants.

Rules:

- `Option[T]` is not implicitly assignable to `T`.
- `T` may be wrapped as `Some(T)` explicitly.
- absence must be handled through matching, narrowing, combinators, or explicit extraction.
- operations such as `map`, `and_then`, `unwrap_or`, and `is_some` may be provided by the runtime support library.

Python APIs that return `T | None` may be adapted at interop boundaries.

## `Result[T, E]`

`Result[T, E]` models recoverable success or failure.

Rules:

- `Ok(T)` and `Err(E)` are distinct variants.
- `Result[T, E]` is not implicitly assignable to `T`.
- postfix `?` may propagate `Err(E)` when the containing function has a compatible result type.
- `map`, `map_err`, `and_then`, `or_else`, `unwrap_or`, `expect`, and related combinators may be supported.
- bare `unwrap()` should be allowed only under an explicit lint/profile policy and should never become the recommended happy path.

## Enums

Rustype enums are closed algebraic sum types.

```python
enum Token:
    Identifier(value: str)
    Integer(value: int)
    End
```

The compiler knows the complete variant set, enabling exhaustiveness checking and precise narrowing.

## Exhaustiveness

A match over a closed Rustype type must account for every reachable variant unless a wildcard arm explicitly covers the remainder.

Adding a new enum variant should cause non-exhaustive matches to fail compilation, making downstream update requirements visible.

## Traits

Traits represent behavioral requirements rather than class ancestry.

The default semantic model should be structural where possible: a type implements a trait when it satisfies the required interface.

Trait definitions may include methods first. Associated types, constants, default method implementations, and specialization should be deferred until their Python lowering and semantics are clear.

## Trait bounds

Generic parameters may be constrained by traits:

```python
fn persist[T: Serializable](value: T) -> Result[None, StorageError]:
    ...
```

Multiple bounds and richer where-style syntax are future design areas.

## Newtypes

A newtype creates a statically distinct type over an existing runtime representation.

```python
newtype UserId = int
newtype OrderId = int
```

`UserId` and `OrderId` are not interchangeable merely because they lower to compatible Python values.

The compiler/runtime design should keep conversion explicit and runtime overhead minimal.

## Typestate

Rustype should make state parameters easy to model through generics and marker types.

```python
class Closed: pass
class Open: pass

class Connection[S]:
    ...
```

Operations can require and return specific states. No ownership semantics are implied; typestate tracks API validity, not resource memory lifetime.

## `Never`

`Never` represents a computation that does not return normally. It is appropriate for panic paths and proven-unreachable branches.

## Dynamic and unknown values

Python interoperability inevitably introduces values whose type cannot be statically trusted.

Rustype should distinguish trusted typed values from dynamic/unknown boundary values. Dynamic values should not silently become trusted values merely because Python permits the operation at runtime.

The compiler should require one of:

- narrowing
- validation
- explicit conversion
- an `unsafe` region

before such values enter Rustype-safe code.

## `Any`

Rustype should avoid treating Python `Any` as a universal silent escape hatch.

Interop may require `Any`, but the Rustype compiler should track or quarantine it so that static guarantees are not silently erased across a module.

The exact representation of "unknown/dynamic" in diagnostics and the internal type lattice is an implementation design decision.

## Type inference

Rustype should infer obvious local types but should favor explicit public APIs.

Initial policy:

- public function parameters require types
- public function return types require types
- local variables may be inferred
- generic inference should be conservative and diagnosable
- compiler inference should never silently introduce an unchecked dynamic type

## Python typing compatibility

Rustype should consume useful Python type information from annotations, stubs, and typed dependencies.

The compiler should not blindly equate Python typing semantics with Rustype guarantees. Python typing information is an interoperability input; Rustype remains responsible for the stronger rules of `.rpy` code.

## Deferred features

Not required for the initial language:

- higher-kinded types
- specialization
- trait coherence rules modeled exactly after Rust
- associated types
- GAT-like features
- ownership-aware variance
- lifetime parameters
- const generics beyond clearly Python-relevant use cases

These should be considered only if concrete Python application patterns justify them.
