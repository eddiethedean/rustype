# Compiler Architecture

Rustype is compiled from `.rpy` source into ordinary Python source by a compiler implemented in Rust.

The canonical compiler representation pipeline is:

```text
.rpy source
   |
   v
lexer/parser
   |
   v
Syntax AST
   |
   v
name resolution + semantic analysis
   |
   v
HIR
(resolved names + semantic types)
   |
   v
lowering
   |
   v
Lowering IR
(explicit control flow + runtime ops)
   |
   v
Python backend
   |
   +--> source map / debug metadata
   |
   v
.py output
   |
   v
CPython
```

The three compiler representations are deliberately separate:

- `AST_MODEL.md` defines syntax-only parser output.
- `HIR_MODEL.md` defines the resolved and typed semantic layer.
- `LOWERING_IR.md` defines backend-oriented explicit control flow.

## Why Rust

Rust is a strong fit for the compiler implementation because it offers:

- fast startup and execution,
- strong enum/AST modeling,
- safe concurrent and incremental infrastructure,
- a single native CLI artifact,
- a shared core suitable for CLI, LSP, and build tooling.

The compiler being written in Rust does not imply Rust runtime semantics for Rustype programs.

## Parser strategy

The parser targets the grammar defined in `GRAMMAR.md` and the normative rules in `V0_SYNTAX_SPEC.md`.

The implementation should evaluate:

1. a dedicated Rustype grammar,
2. reuse of mature Python lexer/parser components where syntax is unchanged,
3. Tree-sitter for editor/incremental parsing,
4. a hand-authored or parser-generator grammar for authoritative compilation.

The authoritative parser must produce precise source spans for every node.

### Parser contract

The parser performs syntax work only.

It must not:

- resolve identifiers,
- infer types,
- decide trait conformance,
- determine match exhaustiveness,
- decide whether `?` means `Result` or `Option` propagation,
- desugar Rustype constructs into Python.

For example:

```python
value = parse()?
```

produces a first-class AST `TryExpr`; it is not expanded during parsing.

## Source representation

Source locations use UTF-8 byte offsets with a per-file line index for line/column conversion.

Every AST node has a source span. HIR and lowering nodes preserve `Origin` metadata. Synthetic compiler nodes point back to the primary `.rpy` span that caused their creation.

This rule applies from the first implementation, not only when source-map support is added.

## Syntax AST

The syntax AST models Rustype source concepts directly rather than prematurely translating into Python AST nodes.

Representative nodes include:

```text
FunctionDecl
TraitDecl
TraitMethod
EnumDecl
VariantDecl
NewtypeDecl
TryExpr (?)
UnsafeBlock
MatchStmt
IfLetStmt
TypeExpr
Pattern
```

The AST is specified in `AST_MODEL.md`.

## HIR

HIR is the semantic authority for valid `.rpy` programs.

It owns:

- symbol tables and scopes,
- imports,
- type name resolution,
- expression types,
- trait identities and structural conformance,
- generic parameters and bounds,
- enum closure and variant membership,
- exhaustive match analysis,
- `?` propagation compatibility,
- safe/unsafe boundaries,
- newtype compatibility,
- dynamic Python boundary classification.

HIR also resolves Rustype intrinsics such as:

```text
Option
Some
None (Option context)
Result
Ok
Err
panic
```

The HIR is specified in `HIR_MODEL.md`.

## Semantic type model

Rustype owns its own semantic type representation rather than delegating language meaning to Python type checkers.

The type model includes dedicated concepts for:

- `Option[T]`,
- `Result[T, E]`,
- enums and variants,
- traits,
- newtypes,
- generic type parameters,
- `Never`,
- ordinary Python-compatible container and callable types,
- dynamic Python values.

Untyped Python interoperability is represented explicitly as a dynamic semantic boundary rather than silently acquiring safe `Any` behavior.

## Type checker

Rustype owns the semantics of Rustype source.

Pyright, BasedPyright, ty, or other Python type checkers may verify generated Python, but they do not define `.rpy` semantics.

This separation prevents Rustype semantics from drifting whenever an external checker changes.

## `?` pipeline

Postfix propagation demonstrates the compiler phase boundaries clearly.

Source:

```python
value = parse()?
```

AST:

```text
Assign(
  value = TryExpr(Call(parse))
)
```

HIR:

```text
Try(
  operand = Call(parse),
  mode = Result { success=T, source_error=E, target_error=E }
)
```

Lowering IR:

```text
t0 = call parse()
if is_err(t0):
    return make_err(extract_err(t0))
value = extract_ok(t0)
```

Only the lowering phase creates the early-return control flow.

## Match pipeline

Source:

```python
match result:
    Ok(value):
        use(value)
    Err(error):
        handle(error)
```

The AST records patterns and arms. HIR resolves `Ok` and `Err`, types bindings, and proves exhaustiveness. Lowering IR then creates variant dispatch.

Generated Python may use pattern matching, class tests, or stable runtime tags; that choice does not affect Rustype semantics.

## Unsafe pipeline

`unsafe:` is a lexical semantic boundary.

The parser records an `UnsafeBlock`. HIR determines which operations are accepted only because the current unsafe depth is nonzero and tags them as unsafe escapes. Lowering normally emits ordinary Python operations while retaining safety metadata for debugging and tooling.

`unsafe` never means raw-memory access.

## Lowering IR

A dedicated lowering representation sits between semantic analysis and Python generation.

Its responsibilities include:

- expansion of `?` into explicit propagation control flow,
- variant dispatch for Rustype matches,
- lowering `if let`,
- generation plans for enum/variant runtime representations,
- trait-compatible Python declarations,
- newtype construction/unwrapping operations,
- explicit runtime helper operations,
- preservation of source-origin metadata.

The IR uses structured blocks and temporaries in v0 rather than requiring SSA.

The IR is specified in `LOWERING_IR.md`.

## Runtime support library

Generated Python may depend on a small `rustype_runtime` package for stable runtime representations such as:

- `Result`, `Ok`, `Err`,
- `Option`, `Some`, empty option representation,
- enum/variant support,
- newtype support,
- panic exception/runtime helper,
- metadata used for debugging/source mapping.

The runtime should be small, pure Python unless a concrete need justifies otherwise, and versioned alongside the compiler contract.

The compiler IR should refer to semantic runtime operations rather than hardcoding runtime helper function names.

## Python code generation

Generated Python should prioritize correctness, traceability, and interoperability over human elegance, but it should remain readable enough to inspect.

Compiler output should be deterministic for reproducible builds and straightforward testing.

The Python backend owns:

- concrete runtime imports,
- generated temporary names,
- Python AST/text formatting,
- declaration ordering,
- source-map emission.

The backend must not reinterpret Rustype semantics.

## Backend verification

During early development, generated Python should optionally or routinely be checked with a mature Python type checker.

Possible pipeline:

```text
rustype check
  -> Rustype parse/HIR/type checks
  -> lower to Python
  -> parse generated Python
  -> optional Pyright/BasedPyright/ty verification
```

Backend verification is a compiler quality gate, not the definition of Rustype semantics.

## Incremental compilation

The architecture should avoid decisions that prevent incremental compilation later.

Parser nodes, symbol tables, type arenas, dependency graphs, and semantic caches should use stable internal identities where practical.

AST `NodeId` values should not be defined as source offsets. Semantic entities should use arena/interned IDs such as `SymbolId`, `TypeId`, `TraitId`, `EnumId`, and `VariantId`.

Incremental compilation becomes especially important for the LSP.

## Compiler invariant checks

Each phase should validate its own output.

### AST invariants

- all nodes have spans,
- blocks are structurally well formed,
- Rustype-only syntax is represented directly,
- no semantic resolution has leaked in.

### HIR invariants

- all identifier references are resolved,
- all expressions have semantic types,
- all `?` expressions have propagation modes,
- all accepted matches are semantically valid,
- all dynamic escapes are classified,
- all trait bounds resolve.

### Lowering IR invariants

- all temporaries are valid,
- all block references exist,
- every runtime operation declares its runtime feature,
- no unresolved AST/HIR ambiguity remains,
- every generated operation has source origin.

Invariant failures after a successful earlier phase are compiler bugs, not user diagnostics.

## Testing strategy

The compiler should use several layers of tests:

- lexer/parser golden tests,
- AST snapshot tests,
- semantic diagnostic tests,
- HIR snapshot tests,
- type-system conformance tests,
- lowering snapshot tests,
- generated-Python parse tests,
- generated-Python execution tests,
- Python interop integration tests,
- source-map/traceback tests,
- differential tests against equivalent handwritten Python where useful.

Each language feature should include both compile-pass and compile-fail fixtures.

The AST, HIR, and lowering test suites should remain distinct so regressions identify the compiler phase that actually changed.

## Compiler correctness principle

If Rustype cannot preserve a feature's intended semantics in Python reliably, the feature should not be admitted to the stable language merely because its syntax is attractive.