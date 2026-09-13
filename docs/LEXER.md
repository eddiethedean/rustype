# Lexer Design

Rustype uses a dedicated Rust lexer before parsing. The lexer is the single source of truth for lexical structure in `.rpy` source.

## Responsibilities

The lexer owns:

- UTF-8 source spans
- identifiers and keywords
- numeric and string literals
- Rustype operators such as postfix `?`
- Python-compatible punctuation
- `NEWLINE`, `INDENT`, and `DEDENT`
- bracket-nesting suppression of logical newlines
- lexical diagnostics

The parser must consume tokens rather than rescan source lines.

## Indentation

Rustype v0 uses Python-style significant indentation.

At logical line starts, indentation changes emit:

```text
INDENT
DEDENT
```

Indentation inside `()`, `[]`, or `{}` does not emit indentation changes or logical newline tokens.

Tabs are rejected in v0 rather than normalized. This avoids ambiguous indentation and keeps formatting deterministic.

## Source spans

Every token carries a byte span into the original UTF-8 `.rpy` source:

```text
Span { file, start, end }
```

Offsets use Rust `usize` and are half-open: `[start, end)`.

AST, HIR, lowering, diagnostics, source maps, and the future LSP should preserve or derive from these original spans.

## Keywords

The lexer recognizes Python-compatible control-flow words plus Rustype-specific words including:

```text
fn
newtype
enum
trait
unsafe
```

`def` is tokenized explicitly so the parser can produce a focused Rustype diagnostic instead of treating it as an unknown identifier.

## Operators

Rustype-specific lexical support includes postfix:

```text
?
```

The lexer does not decide whether `?` is semantically valid. Parsing creates the appropriate expression form and HIR later determines `Result` vs `Option` propagation semantics.

Other recognized operators follow the v0 grammar and Python expression model, including `->`, `:=`, comparisons, arithmetic, bitwise operators, and delimiters.

## Strings

The initial lexer recognizes single-, double-, and triple-quoted strings and reports unterminated literals. Prefix semantics such as `f`, `r`, `b`, and combinations are parser/literal-model work still to be completed; the bootstrap lexer may currently tokenize a prefix identifier separately from the following string.

## Diagnostics

Lexer diagnostic codes use the `RYL` namespace.

Initial codes:

```text
RYL0001  tabs are not allowed
RYL0002  inconsistent indentation
RYL0003  unterminated string literal
RYL0004  unexpected character
```

Lexical diagnostics flow unchanged through `rustype check`, and later through `rustype lsp` and the official VS Code extension.

## Implementation boundary

The intended frontend layering is:

```text
source
  ↓
rustype-lexer
  ↓ token stream
rustype-parser
  ↓ AST
semantic analysis / HIR
```

The parser must not duplicate indentation, comment, string, or operator scanning logic.
