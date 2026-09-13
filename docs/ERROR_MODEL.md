# Error Model

Rustype separates ordinary recoverable failure from programmer errors and broken invariants.

## Recoverable errors: `Result[T, E]`

Expected failures belong in function signatures:

```python
fn parse_user(data: str) -> Result[User, ParseError]:
    ...
```

Callers must handle or propagate the error.

```python
user = parse_user(data)?
```

or:

```python
match parse_user(data):
    Ok(user):
        save(user)
    Err(error):
        report(error)
```

## Optionality: `Option[T]`

Absence without an error uses `Option[T]`:

```python
fn find_user(id: UserId) -> Option[User]:
    ...
```

`Option` should not be used when the absence itself requires domain error information; use `Result` instead.

## Panic

`panic` represents an unrecoverable invariant failure, not routine control flow.

```python
fn impossible_state() -> Never:
    panic("state machine invariant violated")
```

The runtime implementation may raise a dedicated Python exception, but compiler diagnostics and documentation should retain the Rustype distinction between recoverable failure and panic.

## Python exceptions

Python libraries use exceptions extensively. Rustype must interoperate with them without pretending all exception sets are statically knowable.

Initial policy:

- Rustype-defined public APIs should prefer `Result` for expected recoverable failure.
- Python exceptions may cross interop boundaries.
- explicit adapters should convert selected Python exceptions into `Result` values.
- compiler-generated implementation errors should retain source mapping to `.rpy`.

## `unwrap` and `expect`

Rustype runtime types may expose familiar methods:

```python
result.unwrap()
result.expect("configuration was validated at startup")
```

`unwrap()` should be treated as a potentially panicking operation. `expect()` is preferable when a panic represents a documented invariant, but neither should replace proper error handling in routine paths.

Lint policy may later distinguish their severity.

## `?` propagation

Postfix `?` propagates the failure variant of a compatible result-like type.

For `Result[T, E]`:

```python
value = operation()?
```

means conceptually:

```text
Ok(value) -> continue with value
Err(error) -> return Err(error)
```

The compiler must statically verify that propagation is valid for the containing function.

Support for `Option[T]?` should be considered separately and must have unambiguous return semantics.

## Error conversion

Rust-inspired automatic error conversion is attractive, but v1 should be conservative.

Potential future support may allow a declared conversion relationship from `E1` to `E2` when propagating `Result[T, E1]` from a function returning `Result[U, E2]`.

The conversion model must remain explicit and Python-compatible rather than copying Rust's `From` trait mechanically.
