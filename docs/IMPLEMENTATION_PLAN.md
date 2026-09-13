# Implementation Plan

This document turns the roadmap into an initial repository and engineering plan.

## Proposed Rust workspace

```text
rustype/
  Cargo.toml
  crates/
    rustype-cli/
    rustype-parser/
    rustype-ast/
    rustype-semantic/
    rustype-types/
    rustype-lowering/
    rustype-codegen-python/
    rustype-source-map/
    rustype-lsp/              # later phase
  runtime/
    rustype_runtime/
  tests/
    fixtures/
      pass/
      fail/
      interop/
      runtime/
  docs/
```

The exact crate boundaries may change after prototyping; the important rule is keeping syntax, semantics, lowering, and code generation separable.

## First implementation slice

The first slice should prove the complete vertical path with intentionally limited semantics.

Input:

```python
newtype UserId = int

fn greet(id: UserId, name: str) -> str:
    return f"{id}: {name}"
```

Required compiler behavior:

1. parse `.rpy`,
2. build source-spanned AST,
3. resolve names/types,
4. validate the function signature,
5. lower the newtype/function,
6. emit importable Python,
7. execute generated Python in tests,
8. map generated locations back to Rustype source.

Do not begin with every Rust-inspired feature at once.

## Second implementation slice: algebraic values

Add:

- `Option[T]`
- `Some`
- empty option variant
- `Result[T, E]`
- `Ok`
- `Err`
- runtime representation and equality/repr behavior
- type checking for extraction/use

This establishes the runtime/type-system foundation before adding syntactic propagation.

## Third implementation slice: `?`

Implement postfix propagation only after `Result` semantics are stable.

Compiler tasks:

- parse postfix `?`
- infer operand result type
- verify containing function return type
- lower to explicit control flow
- preserve source mapping
- test nested expressions and async functions

Avoid clever Python control-flow tricks; emitted code should be explicit and debuggable.

## Fourth implementation slice: enums and exhaustive matching

Add:

- enum declarations
- unit variants
- tuple/field variants
- construction
- pattern matching
- exhaustive-match analysis
- generated Python representations

This slice should include strong compile-fail coverage because exhaustiveness is a defining guarantee.

## Fifth implementation slice: traits

Start narrowly:

- method signatures only
- structural satisfaction
- generic bound `T: Trait`
- diagnostics for missing/incompatible methods

Defer associated types, default bodies, specialization, and complex coherence rules.

## Python grammar coverage

Rustype should not attempt to reimplement every corner of Python before proving its core features.

Prioritize constructs needed for real application code:

- imports
- assignments
- calls and attributes
- literals/collections
- classes
- decorators
- `if` / loops
- `match`
- exceptions/try for interop
- comprehensions
- async/await
- context managers

Unsupported Python syntax should fail clearly rather than parse incorrectly.

## Runtime package

The first runtime should provide minimal, stable Python implementations for algebraic runtime values.

Potential layout:

```text
rustype_runtime/
  __init__.py
  option.py
  result.py
  enum.py
  panic.py
  debug.py
```

The compiler should generate against an explicit runtime ABI/version so compiler and runtime mismatches can be diagnosed.

## Type engine approach

Build the Rustype type engine incrementally rather than attempting full Python typing compatibility first.

Initial priorities:

1. primitives and classes,
2. functions,
3. generics,
4. Option/Result,
5. enums,
6. newtypes,
7. traits,
8. Python-interoperability types.

The internal type representation should explicitly distinguish Rustype dynamic/unknown values from well-typed values.

## Pyright/BasedPyright/ty usage

Do not make an external Python checker the semantic engine for `.rpy`.

Instead use one or more as development verification:

- verify emitted Python is syntactically/type coherent,
- compare inferred Python-facing APIs,
- catch code-generation regressions,
- provide optional CI backend verification.

Keep this behind an adapter so the backend checker can change.

## Test gates for every feature

Every implemented language feature requires:

- parser pass fixture
- parser failure fixture where relevant
- semantic pass fixture
- semantic failure fixture
- lowering snapshot
- generated-Python parse test
- runtime execution test if observable behavior exists
- source-span/source-map assertion
- interop test where Python-facing behavior is relevant

No language feature is complete when only the happy path transpiles.

## Performance strategy

Do not prematurely optimize code generation. Prioritize correctness and predictable diagnostics.

Architecture should still support:

- incremental parsing
- cached module graphs
- parallel module analysis when safe
- low-overhead native CLI startup

Measure before optimizing.

## Versioning before 1.0

Language syntax and semantics may change before 1.0, but changes should be recorded explicitly.

Maintain:

- language decision log,
- diagnostic code stability where practical,
- runtime ABI version,
- migration notes for syntax changes.

## First milestone definition

The first meaningful public prototype should compile a small multi-module `.rpy` project containing:

- Python imports,
- `fn`,
- `newtype`,
- `Option`,
- `Result`,
- `?`,
- one data-carrying enum,
- exhaustive matching,
- one trait,
- a call into a typed Python package,
- source-mapped diagnostics.

That prototype is enough to validate whether Rustype's central developer experience is genuinely better than typed Python before the project expands further.
