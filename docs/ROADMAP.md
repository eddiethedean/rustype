# Roadmap

Rustype should progress in phases that prove language semantics before investing heavily in polish.

## Phase 0 — Language contract

Goal: stabilize the smallest coherent language worth implementing.

Deliverables:

- grammar decisions for `fn`, `enum`, `trait`, `newtype`, `unsafe`, and postfix `?`
- precise `Result` and `Option` semantics
- exhaustive-match rules
- Python interop rules
- unsafe-boundary rules
- initial runtime representation choices
- diagnostic conventions
- accepted/rejected Rust feature matrix

Exit gate:

A representative corpus of `.rpy` examples can be parsed and their intended lowering is unambiguous on paper.

## Phase 1 — Parser and code generation prototype

Goal: prove `.rpy -> .py` compilation.

Deliverables:

- Rust CLI skeleton
- lexer/token stream with indentation
- parser
- AST with source spans
- Python code emitter
- support for normal Python expressions/statements needed by examples
- `fn`
- `newtype`
- basic `Result` / `Option` runtime
- generated Python execution tests

Exit gate:

Small Rustype programs compile and run correctly on CPython.

## Phase 2 — Core semantic engine

Goal: make Rustype meaningfully safer than syntax sugar.

Deliverables:

- scopes and name resolution
- local type inference
- typed function signatures
- `Result[T, E]`
- `Option[T]`
- postfix `?`
- enum/variant semantics
- exhaustive matching
- newtype compatibility checking
- compile-pass / compile-fail conformance suite

Exit gate:

The compiler catches invalid Result/Option use, newtype confusion, and non-exhaustive matches before Python generation.

## Phase 3 — Traits and Python interop

Goal: support realistic application architecture.

Deliverables:

- traits
- generic parameters and trait bounds
- Python annotation/stub ingestion strategy
- typed Python imports
- unknown/untyped dependency handling
- decorator compatibility
- mixed `.rpy` / `.py` integration tests
- explicit adapters for Python exception boundaries

Exit gate:

A small real application can use FastAPI/Pydantic or another mainstream Python stack from `.rpy` without compromising core Rustype semantics.

## Phase 4 — Unsafe boundaries and validation

Goal: make dynamic Python explicit and containable.

Deliverables:

- `unsafe` semantic model
- tracking of values escaping unsafe regions
- safe wrappers over unsafe internals
- diagnostic rules for unchecked dynamic behavior
- optional validation-boundary integration hooks

Exit gate:

Dynamic Python can be used deliberately while the compiler prevents accidental leakage into trusted Rustype code.

## Phase 5 — Source maps and debugging

Goal: eliminate generated-Python leakage from normal developer workflows.

Deliverables:

- source mapping through lowering
- traceback rewriting
- mixed `.rpy` / `.py` stack traces
- compiler diagnostics with stable codes
- debug metadata design

Exit gate:

Runtime failures in generated code reliably point developers to useful `.rpy` locations.

## Phase 6 — Project tooling

Goal: make Rustype pleasant to use as a project language.

Deliverables:

- `rustype check`
- `rustype build`
- `rustype run`
- `pyproject.toml` configuration
- deterministic output
- incremental build graph
- packaging prototype
- backend Python verification mode

Exit gate:

A Rustype project can be developed and packaged without manually managing generated Python.

## Phase 7 — VS Code linter and editor experience

Goal: make `.rpy` viable for daily development, starting with immediate linting feedback.

Deliverables:

- official VS Code extension
- `.rpy` language registration
- syntax highlighting
- inline lexer/parser/compiler diagnostics
- Problems-panel integration
- stable diagnostic-code presentation
- `rustype explain` integration
- `rustype lsp`
- incremental parser integration
- hover
- completion
- go-to-definition
- references
- rename
- trait navigation
- enum/match code actions
- `Result`/`Option` quick fixes

The extension should remain a thin client over compiler/LSP diagnostics. Lint rules must not be reimplemented in TypeScript.

Exit gate:

Developing a medium-sized `.rpy` project in VS Code feels comparable to modern typed Python tooling, with compiler diagnostics appearing continuously while editing.

## Phase 8 — Stabilization toward 1.0

Goal: freeze a dependable language and toolchain contract.

Deliverables:

- specification cleanup
- compatibility policy
- runtime/compiler versioning contract
- generated-code compatibility rules
- migration tooling for breaking syntax changes before 1.0
- performance profiling
- large integration corpus
- security review of compiler/runtime boundary
- production packaging workflows

Exit gate:

Rustype can commit to language stability and predictable Python interoperability.

## Post-1.0 candidates

Only after the core language proves itself:

- richer trait features
- associated types
- improved error conversion
- `if let`/pattern ergonomics expansion
- immutable local bindings if justified
- deeper Pydantic/msgspec integration
- additional Python checker backends
- formatter
- richer package/build integrations

Ownership, borrowing, lifetimes, and Rust memory semantics remain out of scope even post-1.0 unless the entire project mission is deliberately reconsidered.
