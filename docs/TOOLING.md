# Tooling Plan

Rustype should feel like a modern language toolchain rather than a transpiler users must manually manage.

## CLI

Initial command surface:

```text
rustype check
rustype build
rustype run <entry.rpy>
rustype fmt      # future; only after syntax stabilizes
rustype lsp      # future
rustype explain <diagnostic>
```

The CLI should be a native Rust binary and share compiler crates with the language server.

## Diagnostics

Diagnostics should be compiler-grade:

- stable diagnostic codes
- precise source spans
- concise primary messages
- actionable help text
- related source locations when needed
- clear distinction between Rustype errors and backend/Python errors

Example:

```text
error[RT201]: non-exhaustive match
  --> src/message.rpy:18:1
   |
18 | match message:
   | ^^^^^^^^^^^^^ missing variant `Ping`
```

## Source maps and traceback rewriting

Generated Python line numbers are an implementation detail. Runtime failures should point back to `.rpy` source.

Source mapping must therefore be designed into every lowering step.

Required capabilities:

- generated span -> `.rpy` span mapping
- traceback rewriting for Rustype-generated frames
- preservation of function/module names
- useful stack traces through mixed `.rpy`/`.py` projects
- debugging metadata suitable for IDE integrations

## LSP

A first-class LSP is required before Rustype can be considered production-ready.

Planned capabilities:

- syntax diagnostics
- semantic/type diagnostics
- hover types
- go-to-definition
- find references
- completion
- rename
- trait navigation/implementations
- enum variant completion
- exhaustive-match assistance
- code actions for wrapping in `Ok`, `Err`, `Some`, etc.

The LSP should reuse the compiler lexer, parser, semantic model, source database, and incremental caches rather than implementing a second analyzer.

## VS Code extension and linter

VS Code support is a first-class deliverable rather than a generic-LSP afterthought.

The extension should be intentionally thin: analysis belongs in the Rust compiler/LSP core, while the extension provides editor integration and presentation.

The first useful VS Code milestone is a Rustype linter experience for `.rpy` files:

- register `.rpy` as the Rustype language
- syntax highlighting
- start/manage `rustype lsp`
- publish lexer, parser, and compiler diagnostics inline
- show stable diagnostic codes and source ranges
- Problems-panel integration
- quick links or commands for `rustype explain <code>`
- project/workspace discovery through `pyproject.toml`
- configurable path to the Rustype binary for development installs

Later extension capabilities should include:

- hover and inferred types
- completion
- go-to-definition and references
- rename
- trait implementation navigation
- enum variant completion
- non-exhaustive-match quick fixes
- `Result`/`Option` code actions
- safe/unsafe boundary hints
- run/check/build commands
- traceback navigation back to `.rpy`

The extension should not duplicate lint rules in TypeScript. The Rustype compiler must remain the single source of truth so CLI, CI, VS Code, and every other LSP client report the same diagnostics.

A likely repository layout is:

```text
editors/
    vscode/
        package.json
        syntaxes/
        src/
```

The extension can begin with a TextMate grammar and process-backed diagnostic adapter if necessary, then transition to the shared LSP as soon as `rustype lsp` is available.

## Syntax highlighting

Early editor support can use a Tree-sitter grammar or TextMate grammar while the full LSP is developed. The VS Code extension should ship the first maintained grammar so `.rpy` has useful highlighting from the earliest developer preview.

## Formatter

Rustype should eventually provide one canonical formatter, but implementing it before the grammar stabilizes would waste effort.

Formatter principles should be closer to Black/Ruff formatting than configurable style engines: predictable, low-config output.

## Testing integration

Rustype-generated modules should work with pytest and standard Python testing tools.

Potential convenience command:

```text
rustype test
```

should compile the project and delegate to configured Python test tooling rather than invent a test framework.

## Build system integration

Rustype should integrate with `pyproject.toml` and modern Python packaging.

Potential future components:

- PEP 517 build backend or backend plugin
- editable install support
- `uv` workflows
- wheel inclusion of generated Python and/or Rustype sources
- configurable generated-code directory

## Generated code policy

Generated `.py` should be deterministic and inspectable. Projects should not normally edit it directly.

The toolchain should clearly identify generated files and provide reproducible regeneration.

## Compiler introspection

Developer commands should eventually include:

```text
rustype ast file.rpy
rustype lower file.rpy
rustype emit-python file.rpy
```

These are useful for compiler development, debugging, and advanced users without bloating the normal workflow.
