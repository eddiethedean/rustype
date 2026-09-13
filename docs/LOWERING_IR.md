# Rustype Lowering IR

This document defines the v0 lowering intermediate representation between typed HIR and Python code generation.

The lowering IR exists to make Rustype semantics explicit before generating Python source.

## Why a dedicated IR

The compiler should not generate Python directly from the parser AST or even directly from HIR.

Rustype contains constructs with no direct Python syntax equivalent:

- postfix `?`,
- exhaustive Rustype enum matches,
- Rustype `Option`/`Result` propagation,
- `if let`,
- traits,
- newtypes,
- `unsafe` boundaries,
- source-origin requirements.

A lowering IR gives these features one place to become explicit control flow and runtime operations.

## Pipeline

```text
AST
 |
 v
HIR
(names resolved, types known)
 |
 v
Lowering IR
(explicit control flow and runtime constructs)
 |
 v
Python backend AST/text
 |
 v
.py
```

The lowering IR is backend-oriented, but it should still be independent of Python source formatting.

## Core design rules

The lowering IR must:

1. preserve source origin on every instruction or statement,
2. make hidden control flow explicit,
3. avoid re-running semantic type inference,
4. contain no unresolved identifiers,
5. make runtime helper requirements explicit,
6. be deterministic,
7. be simple enough for snapshot testing.

## Module

```rust
struct LowerModule {
    functions: Vec<LowerFunction>,
    classes: Vec<LowerClass>,
    declarations: Vec<LowerDecl>,
    body: Vec<LowerStmt>,
    runtime_features: RuntimeFeatureSet,
}
```

`runtime_features` records which helpers generated Python must import.

Example flags:

```text
Result
Option
EnumSupport
NewtypeSupport
Panic
SourceMap
```

## Origins

Every lower node carries an `Origin`, inherited from AST/HIR source or synthesized from a primary source span.

Generated temporaries created while lowering:

```python
value = parse()?
```

must still point diagnostics/debug metadata at the `parse()?` expression.

## Values

The IR should use explicit values and temporaries rather than embedding arbitrarily complex expressions everywhere.

Conceptually:

```rust
enum LowerValue {
    Temp(TempId),
    Local(SymbolId),
    Constant(Constant),
    Global(SymbolId),
}
```

Complex expressions lower into instructions that produce temporaries.

## Statements / instructions

Representative operations:

```rust
enum LowerStmt {
    Assign {
        target: Place,
        value: LowerValue,
        origin: Origin,
    },
    Call {
        dest: Option<TempId>,
        callee: LowerValue,
        args: Vec<LowerArg>,
        origin: Origin,
    },
    GetAttr {
        dest: TempId,
        object: LowerValue,
        name: Symbol,
        origin: Origin,
    },
    GetItem {
        dest: TempId,
        object: LowerValue,
        key: LowerValue,
        origin: Origin,
    },
    Branch {
        condition: LowerValue,
        then_block: BlockId,
        else_block: BlockId,
        origin: Origin,
    },
    MatchVariant {
        subject: LowerValue,
        arms: Vec<LowerVariantArm>,
        fallback: Option<BlockId>,
        origin: Origin,
    },
    Return {
        value: Option<LowerValue>,
        origin: Origin,
    },
    Raise {
        value: LowerValue,
        origin: Origin,
    },
    RuntimeOp(RuntimeOp),
}
```

This is a conceptual contract, not necessarily the final Rust enum shape.

## Runtime operations

Rustype-specific lowering uses explicit runtime operations rather than magic calls embedded in strings.

```rust
enum RuntimeOp {
    MakeOk,
    MakeErr,
    MakeSome,
    MakeNone,
    IsOk,
    IsErr,
    IsSome,
    IsNone,
    ExtractOk,
    ExtractErr,
    ExtractSome,
    MakeVariant,
    VariantTag,
    VariantField,
    MakeNewtype,
    UnwrapNewtype,
    Panic,
}
```

The Python backend decides which runtime helper names implement these operations.

## Lowering `Result` propagation

Source:

```python
fn load(path: str) -> Result[Config, ConfigError]:
    text = read_file(path)?
    return parse_config(text)
```

HIR already knows the `?` operand is `Result[str, ConfigError]` and that the enclosing function returns a compatible `Result`.

Lowering IR conceptually becomes:

```text
t0 = call read_file(path)
branch is_err(t0):
    err = extract_err(t0)
    return make_err(err)
else:
    text = extract_ok(t0)

return call parse_config(text)
```

Python might later be generated as:

```python
_rt0 = read_file(path)
if isinstance(_rt0, Err):
    return Err(_rt0.error)
text = _rt0.value
return parse_config(text)
```

The IR must not hardcode this exact representation.

## Error conversion for `?`

v0 should require exact or explicitly supported error compatibility.

The lowering IR should receive any required conversion from HIR as a resolved conversion operation:

```text
Err[E1] --conversion--> Err[E2]
```

No implicit Python exception conversion occurs merely because `?` is used.

If v0 does not support Rust-like `From` conversion initially, the HIR emits only identity-compatible propagation and rejects other cases before lowering.

## Lowering `Option` propagation

Source:

```python
fn manager_name(user: User) -> Option[str]:
    manager = user.manager?
    return Some(manager.name)
```

Lowering:

```text
t0 = get_attr user.manager
branch is_none(t0):
    return make_none()
else:
    manager = extract_some(t0)

...
```

An `Option` `?` may only appear in a function whose return type supports Option propagation under the v0 semantic rules.

## Lowering `if let`

Source:

```python
if let Some(user) = find_user(id):
    print(user.name)
else:
    missing()
```

HIR resolves `Some` as an Option pattern.

Lowering:

```text
t0 = call find_user(id)
branch is_some(t0):
    user = extract_some(t0)
    call print(user.name)
else:
    call missing()
```

For enum variants, the same form uses variant-tag tests and field extraction.

## Lowering exhaustive `match`

Source:

```python
match result:
    Ok(value):
        use(value)
    Err(error):
        handle(error)
```

HIR performs exhaustiveness checking before lowering.

The IR can represent this with a typed variant dispatch:

```text
match_variant result:
    Ok(value) -> block_1
    Err(error) -> block_2
```

The backend may emit Python `match`, `isinstance`, tag checks, or another stable implementation.

**Important:** generated Python does not need to preserve Rustype syntax. It needs to preserve Rustype semantics and source mapping.

## Match guards

A Rustype match arm with a guard:

```python
match value:
    Some(x) if x > 0:
        ...
    Some(_):
        ...
    None:
        ...
```

lowers to variant dispatch followed by a guard branch within the relevant variant path.

Exhaustiveness analysis occurs over patterns independent of guard success. A guarded arm does not generally cover the full variant unless another arm covers the remainder.

## Enums

A v0 Rustype enum:

```python
enum Message:
    Quit
    Move(x: int, y: int)
```

lowers into a backend declaration plan rather than ordinary statements.

Conceptually:

```rust
struct LowerEnum {
    id: EnumId,
    name: Symbol,
    variants: Vec<LowerVariantDecl>,
    origin: Origin,
}
```

Each variant contains:

- stable enum identity,
- variant identity/tag,
- field names and order,
- source origin,
- generated Python name.

The initial Python representation should favor:

- immutability,
- structural pattern compatibility where practical,
- readable `repr`,
- stable equality semantics,
- easy introspection,
- no dependence on Rust memory layout.

A likely backend is generated frozen dataclasses or equivalent runtime-supported classes.

## `Option` and `Result`

`Option` and `Result` are semantic intrinsics even if represented by runtime classes.

HIR should resolve:

```text
Some(...)
None
Ok(...)
Err(...)
```

into intrinsic variant construction, so lowering never has to guess whether a user-defined function named `Ok` is meant.

Shadowing rules must be specified separately; v0 should strongly consider reserving these constructors in type-safe Rustype contexts.

## Newtypes

Source:

```python
newtype UserId = int
```

HIR knows `UserId` is semantically distinct from `int`.

Lowering should treat construction and unwrapping explicitly:

```text
make_newtype UserId(value)
unwrap_newtype UserId(value)
```

The Python runtime representation is intentionally backend-specific.

For v0, the backend should prefer a lightweight wrapper with stable runtime identity rather than erasing the type completely, because:

- debugging is clearer,
- Python interop can distinguish newtypes,
- accidental runtime mixing is reduced,
- source-level semantics are easier to preserve.

This can be revisited after benchmarking.

## Traits

Traits are compile-time behavioral contracts.

Source:

```python
trait Repository:
    fn get(self, id: UserId) -> Option[User]
```

Lowering should normally generate Python typing/interoperability constructs, likely a `typing.Protocol`-compatible declaration.

No Rust-style vtable or trait-object runtime is required in v0.

Trait conformance is proven during semantic analysis. Lowering only needs sufficient generated structure for:

- Python introspection,
- generated annotations,
- external type checkers,
- optional runtime-checkable behavior if explicitly requested later.

## `unsafe`

`unsafe` is primarily a semantic-analysis construct.

By the time code reaches lowering IR, operations permitted only because they occurred inside `unsafe` should be marked in metadata:

```rust
Safety::Safe
Safety::UnsafeEscape
```

The generated Python generally executes the underlying operation normally.

The backend does **not** emit memory-unsafe code or special memory instructions.

The safety marker is useful for:

- debug metadata,
- auditing,
- tooling,
- future `rustype explain` output.

## Python dynamic calls

A dynamic call inside `unsafe` such as:

```python
unsafe:
    handler = getattr(module, name)
    handler(data)
```

lowers to ordinary attribute lookup and call operations tagged as originating from an unsafe escape.

Outside `unsafe`, HIR should reject an operation that violates the active Rustype safety rules before it reaches lowering.

## Exceptions

Python `try` / `except` remains distinct from Rustype `Result`.

The lowering IR must not silently turn arbitrary Python exceptions into `Err` values.

Explicit adapters may later lower operations such as:

```text
catch_exception -> map_to_error -> make_err
```

but only when source syntax or a recognized library API requests it.

## Async

`async fn` lowers to a Python async function.

`await` remains an explicit lower operation.

`?` inside `async fn` behaves exactly like in sync functions with respect to `Result`/`Option`; it does not change await semantics.

Example:

```python
async fn load() -> Result[Data, Error]:
    response = await fetch()?
    return Ok(response.data)
```

parses so that `?` applies to the awaited result according to grammar precedence. HIR determines the exact operand type before lowering.

## Temporaries

Compiler-generated temporaries use IDs, not user-visible names:

```rust
struct TempId(u32);
```

Python name allocation happens only in the backend:

```text
TempId(0) -> _rustype_t0
```

Generated names must avoid collision with user names deterministically.

## Places

Assignment targets should be explicit:

```rust
enum Place {
    Local(SymbolId),
    Attribute {
        object: LowerValue,
        name: Symbol,
    },
    Subscript {
        object: LowerValue,
        key: LowerValue,
    },
    Tuple(Vec<Place>),
}
```

This avoids treating assignment syntax as ordinary expressions.

## Control-flow graph

The initial implementation may represent lower IR as structured blocks rather than a full SSA graph.

However, blocks should already have stable IDs:

```rust
struct LowerBlock {
    id: BlockId,
    statements: Vec<LowerStmt>,
}
```

This leaves a clean path to CFG-based analysis later without requiring SSA in v0.

## Python backend responsibilities

The Python backend owns:

- selecting concrete runtime helper names,
- import generation,
- temporary variable names,
- Python AST/text formatting,
- source-map emission,
- deterministic declaration ordering,
- runtime-support version metadata.

It must not:

- reinterpret Rustype types,
- decide match exhaustiveness,
- resolve traits,
- decide whether `?` is valid,
- infer safety boundaries.

Those decisions are already complete before code generation.

## Source maps

Every generated Python statement should map to one primary `.rpy` origin.

Where one Rustype expression expands into several Python statements, every generated statement should reference the same primary source span plus an optional synthetic reason.

Example:

```python
value = parse()?
```

may produce an assignment, test, early return, and extraction. All four generated regions map back to the `parse()?` span, with the final assignment optionally mapping more narrowly to the original assignment target/value.

## Lowering validation

Before code generation, the compiler should validate IR invariants:

- all blocks referenced exist,
- every temporary is defined before use within valid control flow,
- every runtime operation declares a required runtime feature,
- every node has an origin,
- no unresolved symbol/type IDs remain,
- no raw AST nodes remain,
- no unresolved `TryExpr` mode remains,
- enum dispatch references real variants,
- newtype operations reference real newtypes.

These checks should fail as compiler bugs, not user diagnostics.

## Snapshot examples

The test suite should snapshot both HIR and lowering IR for representative features.

Required v0 lowering fixtures include:

- `Result` `?`,
- `Option` `?`,
- nested `?`,
- `if let Some`,
- `if let` enum variant,
- exhaustive enum match,
- guarded match,
- newtype construction/use,
- trait-typed parameters,
- unsafe reflection,
- async + await + `?`,
- mixed `.rpy`/`.py` calls,
- source-span preservation across all generated control flow.