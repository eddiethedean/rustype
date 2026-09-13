# Compiler Architecture

Rustype is compiled from `.rpy` source into ordinary Python source by a compiler implemented in Rust.

## High-level pipeline

```text
.rpy source
   |
   v
lexer/parser
   |
   v
Rustype AST
   |
   v
name resolution
   |
   v
semantic analysis
   |
   v
type checking
   |
   v
lowering IR
   |
   v
Python code generation
   |
   +--> source map / debug metadata
   |
   v
.py output
   |
   v
CPython
```

## Why Rust

Rust is a strong fit for the compiler implementation because it offers:

- fast startup and execution
- strong enum/AST modeling
- safe concurrent and incremental infrastructure
- a single native CLI artifact
- a shared core suitable for CLI, LSP, and build tooling

The compiler being written in Rust does not imply Rust runtime semantics for Rustype programs.

## Parser strategy

The parser should target a Python-like grammar extended with Rustype constructs such as `fn`, `trait`, `enum`, `newtype`, postfix `?`, and `unsafe`.

The implementation should evaluate:

1. a dedicated Rustype grammar,
2. reuse of mature Python lexer/parser components where syntax is unchanged,
3. Tree-sitter for editor/incremental parsing,
4. a hand-authored or parser-generator grammar for authoritative compilation.

The compiler's canonical parser must produce precise source spans for every node.

## AST

The Rustype AST should model source concepts directly rather than prematurely translating everything into Python AST nodes.

Examples:

```text
Function
Trait
TraitMethod
Enum
Variant
Newtype
TryExpr (?)
UnsafeBlock
Match
IfLet
ResultType
OptionType
```

This keeps semantic rules understandable and prevents Python lowering details from leaking into the language model.

## Semantic analysis

Semantic analysis should cover:

- symbol tables and scopes
- imports
- type name resolution
- trait resolution
- generic parameters and bounds
- enum closure and variant membership
- exhaustive match analysis
- `?` propagation compatibility
- safe/unsafe boundaries
- newtype compatibility
- typestate-relevant generic checking

## Type checker

Rustype should eventually own the semantics of Rustype source.

Pyright/BasedPyright/ty may be useful as Python-backend verification tools, but they should not define the language specification.

This separation prevents Rustype semantics from drifting whenever an external checker changes.

## Lowering IR

A dedicated lowering representation should sit between semantic analysis and Python generation.

Its responsibilities include:

- expansion of `?` into explicit result propagation
- generation of enum/variant runtime representations
- generation of trait-compatible Python constructs
- lowering newtypes
- transforming Rustype match semantics into Python-compatible code
- preserving source-origin metadata

The IR should make backend transformations testable without coupling them to parser structures.

## Runtime support library

Generated Python may depend on a small `rustype_runtime` package for stable runtime representations such as:

- `Result`, `Ok`, `Err`
- `Option`, `Some`, empty option representation
- variant support
- panic exception/runtime helper
- metadata used for debugging/source mapping

The runtime should be small, pure Python unless a concrete need justifies otherwise, and versioned alongside the compiler contract.

## Python code generation

Generated Python should prioritize correctness, traceability, and interoperability over human elegance, but it should remain readable enough to inspect.

Compiler output should be deterministic for reproducible builds and straightforward testing.

## Backend verification

During early development, generated Python should optionally or routinely be checked with a mature Python type checker.

Possible pipeline:

```text
rustype check
  -> Rustype semantic/type checks
  -> lower to Python
  -> parse generated Python
  -> optional Pyright/BasedPyright/ty verification
```

Backend verification is a compiler quality gate, not the definition of Rustype semantics.

## Incremental compilation

The architecture should avoid decisions that prevent incremental compilation later. Parser nodes, symbol tables, dependency graphs, and semantic caches should have stable identities where practical.

Incremental compilation becomes especially important for the LSP.

## Testing strategy

The compiler should use several layers of tests:

- lexer/parser golden tests
- AST snapshot tests
- semantic diagnostic tests
- type-system conformance tests
- lowering snapshot tests
- generated-Python execution tests
- Python interop integration tests
- source-map/traceback tests
- differential tests against equivalent handwritten Python where useful

Each language feature should include both compile-pass and compile-fail fixtures.

## Compiler correctness principle

If Rustype cannot preserve a feature's intended semantics in Python reliably, the feature should not be admitted to the stable language merely because its syntax is attractive.
