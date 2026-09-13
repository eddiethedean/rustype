# Language Design

This document defines the initial design rules for the `.rpy` language.

## Core rule

Rustype borrows Rust syntax only when doing so gives Python code a clearer static guarantee, safer state model, or more explicit control-flow contract.

Rustype should not become "Rust with indentation." Python remains the host mental model and runtime model.

## Source files

Rustype source files use the `.rpy` extension.

```text
src/
  app.rpy
  domain.rpy
  legacy.py
```

A project may freely mix `.rpy` and `.py` modules.

## Functions

Rustype uses `fn` and `async fn` for Rustype-defined functions.

```python
fn add(a: int, b: int) -> int:
    return a + b

async fn fetch_user(id: UserId) -> Result[User, FetchError]:
    ...
```

Using `fn` gives `.rpy` a visible language boundary and creates room for Rustype-specific semantics without silently changing the meaning of ordinary Python `def`.

Whether plain `def` is accepted as an escape/interoperability form remains an open design decision.

## Newtypes

Rustype provides first-class newtypes:

```python
newtype UserId = int
newtype AccountId = int
```

`UserId` and `AccountId` are statically distinct even though both lower to Python-compatible runtime representations.

## Option

Optional values use Rustype's `Option[T]` model.

```python
fn find_user(id: UserId) -> Option[User]:
    ...
```

Construction and matching use Rust-inspired variants:

```python
return Some(user)
return None
```

The parser/compiler owns the distinction between the option `None` variant and Python's runtime `None`; lowering may use internal runtime helpers where necessary.

An `Option[T]` cannot be used as `T` without narrowing, matching, or an explicit operation such as `unwrap_or`.

## Result

Recoverable failures use `Result[T, E]`.

```python
fn parse_port(value: str) -> Result[int, ParseError]:
    ...
```

Values are constructed with:

```python
Ok(8000)
Err(ParseError(...))
```

A `Result[T, E]` cannot be implicitly treated as `T`.

## Error propagation

Rustype reserves postfix `?` for propagation of compatible `Result` and potentially `Option` values.

```python
fn load_config(path: str) -> Result[Config, ConfigError]:
    text = read_file(path)?
    return parse_config(text)
```

The compiler lowers this to explicit Python control flow while preserving `.rpy` source positions.

## Enums and variants

Rustype enums are closed sum types and may carry data.

```python
enum Message:
    Quit
    Move(x: int, y: int)
    Write(value: str)
```

The compiler should lower variants to frozen Python-compatible representations and preserve variant identity for pattern matching and Python interop.

## Pattern matching

Pattern matching should remain close to Python's `match`, but matching over Rustype closed types is exhaustive by default.

```python
match message:
    Quit:
        stop()
    Move(x, y):
        move_to(x, y)
    Write(value):
        print(value)
```

Missing a reachable variant is a compile-time error unless an explicit wildcard/default arm is permitted by the active rule set.

## `if let`

Rustype may support Rust-inspired single-pattern narrowing:

```python
if let Some(user) = find_user(id):
    print(user.name)
```

This feature is justified because it provides concise, statically understood narrowing for algebraic types.

## Traits

Traits define behavioral capabilities.

```python
trait Repository:
    fn get(self, id: UserId) -> Option[User]
```

The Python lowering should prefer structural compatibility, most likely through `typing.Protocol` or generated equivalents, rather than requiring nominal inheritance.

## Generic trait bounds

```python
fn load[R: Repository](repo: R, id: UserId) -> Option[User]:
    return repo.get(id)
```

Rustype should reuse Python's runtime generic model wherever practical while enforcing stronger static rules in the compiler.

## Unsafe regions

Dynamic operations that bypass Rustype guarantees are made explicit:

```python
unsafe:
    module = dynamic_import(name)
    handler = getattr(module, "handler")
```

Unsafe is not about memory safety. In Rustype it marks **static-safety escape boundaries**: reflection, unchecked dynamic typing, untyped foreign APIs, runtime mutation, and related behavior.

## Panic and Never

Rustype distinguishes recoverable failure from unrecoverable failure.

```python
fn impossible() -> Never:
    panic("unreachable state")
```

Routine recoverable errors should use `Result`; `panic` represents broken invariants or unrecoverable conditions.

## Typestate

Rustype encourages state transitions in types:

```python
class Draft:
    pass

class Published:
    pass

class Document[S]:
    ...

fn publish(doc: Document[Draft]) -> Document[Published]:
    ...
```

The initial implementation can build typestate on normal generic type parameters and marker types; a dedicated syntax is not required for v1.

## Immutability

Rustype should support and encourage immutable domain values where useful, but it must not pretend Python objects have Rust ownership semantics.

Immutability is a type/design guarantee, not a memory model.

## Decorators and Python syntax

Ordinary Python constructs such as decorators should remain available:

```python
@app.get("/users/{id}")
async fn get_user(id: UserId) -> Result[User, UserError]:
    ...
```

The general rule is compatibility unless a construct prevents Rustype from providing its promised guarantees.

## Design filter for new syntax

A proposed Rust-derived feature should be accepted only if all of the following are true:

1. It provides a meaningful static or modeling benefit.
2. Existing Python syntax cannot express the concept as clearly.
3. It can lower predictably to Python semantics.
4. It does not depend on Rust ownership/borrowing semantics.
5. It remains understandable to a Python developer.
6. It does not unnecessarily damage Python library interoperability.
