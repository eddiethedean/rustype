# Rustype HIR Model

This document defines the v0 high-level intermediate representation (HIR), the semantic layer between the parser AST and the lowering IR.

HIR is where Rustype source acquires **meaning**.

```text
AST                  HIR                         Lowering IR
syntax only   ->      resolved + typed     ->    explicit control flow
```

## Responsibilities

HIR owns:

- symbol resolution,
- lexical scopes,
- import resolution,
- type resolution and inference,
- generic parameter identities,
- trait identities and conformance,
- enum and variant identities,
- newtype identities,
- `Option`/`Result` intrinsic resolution,
- `?` propagation mode,
- match-pattern typing,
- exhaustiveness results,
- safe/unsafe classification,
- semantic diagnostics.

HIR does **not** own:

- Python helper names,
- generated temporary names,
- Python source formatting,
- runtime representation choices,
- explicit desugaring of `?` into branches.

## Stable IDs

Semantic entities use interned IDs:

```rust
struct HirId(u32);
struct SymbolId(u32);
struct ScopeId(u32);
struct TypeId(u32);
struct FunctionId(u32);
struct ClassId(u32);
struct EnumId(u32);
struct VariantId(u32);
struct TraitId(u32);
struct NewtypeId(u32);
```

IDs are compiler-internal identities, not source offsets.

## HIR node envelope

Expressions and statements retain source origin and semantic type where applicable.

```rust
struct HirExpr {
    id: HirId,
    origin: Origin,
    ty: TypeId,
    safety: Safety,
    kind: HirExprKind,
}

enum Safety {
    Safe,
    UnsafeEscape,
}
```

`UnsafeEscape` records that an operation is accepted only because it occurs within an explicit Rustype `unsafe` region or another specified dynamic boundary.

## Semantic type model

The v0 type arena should represent types independently from source syntax.

Conceptually:

```rust
enum TypeKind {
    Unknown,
    Never,
    NoneType,
    Bool,
    Int,
    Float,
    Str,
    Bytes,
    Class(ClassId, Vec<TypeId>),
    Enum(EnumId, Vec<TypeId>),
    Trait(TraitId, Vec<TypeId>),
    Newtype(NewtypeId),
    Option(TypeId),
    Result { ok: TypeId, err: TypeId },
    Tuple(Vec<TypeId>),
    List(TypeId),
    Set(TypeId),
    Dict { key: TypeId, value: TypeId },
    Callable(CallableType),
    Union(Vec<TypeId>),
    TypeParam(SymbolId),
    PythonDynamic,
}
```

`PythonDynamic` represents values whose static behavior Rustype cannot prove, such as untyped Python interop. It is deliberately not treated as a universal safe supertype.

## Dynamic values

Rustype should distinguish:

- `object`-like values that are statically known but broad,
- unresolved compiler states (`Unknown`, internal only),
- intentionally dynamic Python values (`PythonDynamic`).

`PythonDynamic` operations outside an allowed boundary should produce diagnostics rather than silently behaving like Python `Any`.

This is central to Rustype's safety model.

## Symbols and namespaces

HIR name resolution should track value and type namespaces explicitly where needed.

A symbol records:

```rust
struct Symbol {
    id: SymbolId,
    name: SymbolName,
    kind: SymbolKind,
    declared_at: Origin,
    visibility: Visibility,
}
```

Representative kinds:

```rust
enum SymbolKind {
    Module,
    Import,
    Local,
    Parameter,
    Function(FunctionId),
    Class(ClassId),
    Enum(EnumId),
    Variant(VariantId),
    Trait(TraitId),
    Newtype(NewtypeId),
    TypeParameter,
}
```

## Functions

```rust
struct HirFunction {
    id: FunctionId,
    origin: Origin,
    name: SymbolId,
    is_async: bool,
    generics: Vec<HirGenericParam>,
    params: Vec<HirParam>,
    return_type: TypeId,
    body: HirBlock,
}
```

v0 Rustype functions should require a known return type after semantic analysis. If source syntax omits one and v0 does not permit inference for that location, HIR construction fails with a user diagnostic.

## Generic bounds

```rust
struct HirGenericParam {
    symbol: SymbolId,
    bounds: Vec<TraitId>,
}
```

Although v0 syntax initially permits one bound form (`T: Trait`), HIR stores a list so semantic representation does not need redesign when multiple bounds are added later.

## Trait conformance

Traits are structural in v0.

HIR conformance checking compares the required method set and signatures of a trait against the available members of a candidate type.

The compiler should cache conformance results by `(TypeId, TraitId)`.

No explicit Rust-style `impl Trait for Type` declaration is required in v0.

## Intrinsics

Rustype built-ins are resolved semantically, not by textual spelling during lowering.

Representative intrinsic identities:

```rust
enum Intrinsic {
    Option,
    Some,
    OptionNone,
    Result,
    Ok,
    Err,
    Panic,
}
```

This allows HIR to distinguish Rustype `Ok(...)` from a Python or user-defined function with a similar spelling according to the language's shadowing rules.

## Variant construction

```rust
enum HirExprKind {
    Local(SymbolId),
    Literal(HirLiteral),
    Call(HirCall),
    VariantConstruct {
        variant: VariantRef,
        args: Vec<HirExpr>,
    },
    Try(HirTryExpr),
    Attribute(HirAttribute),
    ...
}
```

```rust
enum VariantRef {
    Enum(VariantId),
    OptionSome,
    OptionNone,
    ResultOk,
    ResultErr,
}
```

## `?` semantic representation

```rust
struct HirTryExpr {
    operand: Box<HirExpr>,
    mode: PropagationMode,
}

enum PropagationMode {
    Result {
        success: TypeId,
        source_error: TypeId,
        target_error: TypeId,
        conversion: ErrorConversion,
    },
    Option {
        success: TypeId,
    },
}
```

```rust
enum ErrorConversion {
    Identity,
    // future: FromTrait(...)
}
```

In v0, `Result` propagation should normally require `Identity` error compatibility unless another conversion rule is explicitly specified in the language spec.

### Validation

HIR creation rejects `?` when:

- the operand is neither `Result` nor `Option`,
- `Result` is used in a function without a compatible `Result` return type,
- `Option` is used in a function without a compatible `Option` return type,
- the error type cannot be propagated,
- `?` occurs where early return is impossible.

Lowering never needs to rediscover these facts.

## Patterns

Patterns become typed semantic patterns:

```rust
enum HirPattern {
    Wildcard,
    Binding {
        symbol: SymbolId,
        ty: TypeId,
    },
    Literal(HirLiteral),
    Variant {
        variant: VariantRef,
        fields: Vec<HirPattern>,
    },
    Tuple(Vec<HirPattern>),
    Or(Vec<HirPattern>),
}
```

Pattern bindings create symbols scoped to the relevant match arm or `if let` body.

## Match semantics

```rust
struct HirMatch {
    subject: HirExpr,
    arms: Vec<HirMatchArm>,
    exhaustiveness: Exhaustiveness,
}
```

```rust
enum Exhaustiveness {
    Exhaustive,
    NonExhaustive {
        missing: Vec<MissingPattern>,
    },
}
```

A `NonExhaustive` result normally emits a compile-time diagnostic and prevents lowering.

HIR should retain the missing-pattern analysis so diagnostics can say what is absent, e.g.:

```text
non-exhaustive match: missing `Database(_)`
```

## Guards

A guarded arm narrows according to its pattern within the arm body, but guard presence affects coverage analysis.

Example:

```python
Some(x) if x > 0:
```

does not cover all `Some` values.

## `if let`

HIR normalizes `if let` as its own semantic statement rather than immediately rewriting it into a match.

```rust
struct HirIfLet {
    pattern: HirPattern,
    value: HirExpr,
    then_block: HirBlock,
    else_block: Option<HirBlock>,
}
```

This preserves source intent for diagnostics and tooling. Lowering may then transform it to branch/variant-test operations.

## Newtypes

```rust
struct HirNewtype {
    id: NewtypeId,
    symbol: SymbolId,
    underlying: TypeId,
    origin: Origin,
}
```

Newtypes are nominally distinct:

```text
UserId != int
UserId != AccountId
```

even if they share the same underlying type.

Construction is represented semantically:

```rust
HirExprKind::NewtypeConstruct {
    newtype: NewtypeId,
    value: Box<HirExpr>,
}
```

Implicit conversion from the underlying type into a newtype should be rejected in v0 unless explicitly allowed by a future rule.

## Unsafe semantics

The HIR builder maintains an unsafe-depth context.

```text
unsafe_depth = 0  -> safe context
unsafe_depth > 0  -> unsafe escape context
```

Entering `unsafe:` increments depth for its lexical body.

Dynamic operations that require unsafe produce either:

- a diagnostic when depth is zero, or
- an HIR node tagged `Safety::UnsafeEscape` when depth is nonzero.

Nesting unsafe blocks is semantically harmless but retains source structure.

## Python interop

Imports from normal `.py` modules may produce fully typed or dynamic symbols depending on available type information.

Examples of evidence the semantic engine may use later:

- Python annotations,
- `.pyi` stubs,
- `py.typed`,
- recognized standard-library metadata,
- Rustype-generated interface metadata.

Untyped Python APIs should resolve to `PythonDynamic`, not silently become safe Rustype types.

## Exceptions versus Result

HIR keeps Python exception control flow and `Result` values distinct.

A call returning `Result[T, E]` is ordinary typed value flow.

A Python operation that may raise does not automatically acquire `Result` type.

Explicit adapters, if added, must have dedicated semantic representation.

## Typestate

v0 typestate is expressed through ordinary generic types and marker classes/newtypes.

HIR does not need a dedicated `Typestate` node. The effect emerges from generic type checking:

```python
Connection[Disconnected]
Connection[Connected]
```

The compiler should avoid special-casing typestate until a concrete feature requires it.

## HIR invariants

After HIR construction succeeds:

- every identifier reference is resolved,
- every expression has a `TypeId`,
- every `?` has a known `PropagationMode`,
- every Rustype variant reference has stable identity,
- every pattern is typed,
- every accepted match is exhaustive or explicitly allowed by language rules,
- every dynamic escape is classified,
- every generic bound resolves to a trait,
- every newtype is nominally identified,
- no parser-only ambiguity remains.

Failure to meet one of these conditions is either a user-facing semantic error or a compiler bug.

## Diagnostics

HIR should emit diagnostics with:

- stable diagnostic code,
- primary `.rpy` span,
- concise message,
- zero or more secondary spans,
- optional structured fix hint.

Examples:

```text
RYP2001: `?` requires Result or Option
RYP2002: cannot propagate `DatabaseError` as `ConfigError`
RYP2101: non-exhaustive match; missing `None`
RYP2201: dynamic attribute access requires `unsafe`
RYP2301: `UserId` is not assignable from `int`
RYP2401: type does not satisfy trait `Repository`
```

Exact code allocation can be finalized separately, but semantic errors should be designed for stable tooling consumption from the start.

## Testing requirements

HIR tests should use source fixtures and assert:

- resolved symbols,
- semantic types,
- propagation modes,
- enum/variant resolution,
- trait conformance,
- newtype incompatibility,
- exhaustiveness results,
- pattern binding scopes,
- unsafe classification,
- Python dynamic-boundary diagnostics.

Parser snapshots and HIR snapshots must remain separate so semantic changes do not masquerade as grammar changes.