# Rustype v0 AST Model

This document defines the canonical abstract syntax tree produced by the authoritative Rustype parser for v0.

The AST models **Rustype source concepts**, not generated Python. Python lowering is a later compiler phase.

## Design goals

The AST must:

- preserve exact source origin for every meaningful node,
- represent Rustype-specific constructs directly,
- remain independent of the Python backend,
- support stable diagnostics and future incremental analysis,
- separate syntax from semantic/type information,
- make parser tests deterministic and snapshot-friendly.

## Source identity

Every syntax node carries a stable node identifier and a source span.

Conceptually:

```rust
struct NodeId(u32);

struct Span {
    file: FileId,
    start: TextOffset,
    end: TextOffset,
}
```

`Span` uses half-open byte offsets `[start, end)` in UTF-8 source. Line/column data is derived through a per-file line index rather than duplicated on every node.

Compiler-generated nodes must retain an `Origin`:

```rust
enum Origin {
    Source(Span),
    Synthetic {
        primary: Span,
        reason: SyntheticReason,
    },
}
```

This is required so lowering, diagnostics, and generated traceback metadata always point back to `.rpy`.

## AST layering

Rustype uses three representations:

1. **Syntax AST** — direct parser output.
2. **HIR** (high-level semantic IR) — names resolved and types attached.
3. **Lowering IR** — Python-oriented explicit control flow.

The parser produces only the syntax AST.

```text
source -> AST -> HIR -> lowering IR -> Python
```

The AST must not contain resolved symbols, inferred types, exhaustiveness results, or Python code-generation decisions.

## Module

```rust
struct Module {
    id: NodeId,
    span: Span,
    items: Vec<Item>,
}
```

A module is a `.rpy` file.

## Top-level items

```rust
enum Item {
    Import(ImportStmt),
    FromImport(FromImportStmt),
    Function(FunctionDecl),
    Class(ClassDecl),
    Enum(EnumDecl),
    Trait(TraitDecl),
    Newtype(NewtypeDecl),
    Statement(Stmt),
}
```

Top-level executable statements remain allowed where ordinary Python allows them unless forbidden by another v0 rule.

## Identifiers

Identifiers are stored as interned names plus source span.

```rust
struct Ident {
    id: NodeId,
    span: Span,
    name: Symbol,
}
```

The parser does not determine whether an identifier refers to a value, type, trait, variant, import, or generic parameter.

## Functions

```rust
struct FunctionDecl {
    id: NodeId,
    span: Span,
    decorators: Vec<Expr>,
    is_async: bool,
    name: Ident,
    generics: Vec<GenericParam>,
    params: Vec<Param>,
    return_type: Option<TypeExpr>,
    body: Block,
}
```

`fn` and `async fn` both map to `FunctionDecl`; `is_async` records the distinction.

```rust
struct Param {
    id: NodeId,
    span: Span,
    kind: ParamKind,
    name: Ident,
    annotation: Option<TypeExpr>,
    default: Option<Expr>,
}

enum ParamKind {
    PositionalOnly,
    PositionalOrKeyword,
    VarArgs,
    KeywordOnly,
    VarKwargs,
}
```

The parser preserves Python-compatible parameter structure. Semantic analysis enforces Rustype annotation requirements.

## Generic parameters

v0 supports simple generic parameters and trait bounds.

```rust
struct GenericParam {
    id: NodeId,
    span: Span,
    name: Ident,
    bound: Option<TypeExpr>,
}
```

Example:

```python
fn load[R: Repository](repo: R) -> Option[User]:
    ...
```

produces one generic parameter `R` with bound `Repository`.

## Newtypes

```rust
struct NewtypeDecl {
    id: NodeId,
    span: Span,
    name: Ident,
    underlying: TypeExpr,
}
```

Example:

```python
newtype UserId = int
```

No runtime representation decision appears in the AST.

## Enums

```rust
struct EnumDecl {
    id: NodeId,
    span: Span,
    name: Ident,
    generics: Vec<GenericParam>,
    variants: Vec<VariantDecl>,
}
```

```rust
struct VariantDecl {
    id: NodeId,
    span: Span,
    name: Ident,
    fields: VariantFields,
}

enum VariantFields {
    Unit,
    Positional(Vec<TypeExpr>),
    Named(Vec<VariantField>),
}
```

For v0 source syntax, named fields are canonical:

```python
enum UserError:
    NotFound(id: UserId)
    Database(message: str)
```

`Unit` represents variants such as `Quit`.

The `Positional` form is retained in the AST design to avoid blocking a future tuple-variant syntax, but the v0 parser need not accept it unless the syntax spec explicitly does.

## Traits

```rust
struct TraitDecl {
    id: NodeId,
    span: Span,
    name: Ident,
    generics: Vec<GenericParam>,
    methods: Vec<TraitMethod>,
}
```

```rust
struct TraitMethod {
    id: NodeId,
    span: Span,
    is_async: bool,
    name: Ident,
    generics: Vec<GenericParam>,
    params: Vec<Param>,
    return_type: Option<TypeExpr>,
}
```

Trait methods have signatures only in v0. Default method implementations are deferred.

## Classes

Normal Python-style classes remain separate from traits and enums.

```rust
struct ClassDecl {
    id: NodeId,
    span: Span,
    decorators: Vec<Expr>,
    name: Ident,
    generics: Vec<GenericParam>,
    bases: Vec<Expr>,
    body: Block,
}
```

A Rustype enum is **not** represented as a class hierarchy in the AST even if the Python backend eventually emits classes.

## Statements

```rust
enum Stmt {
    Assign(AssignStmt),
    AnnAssign(AnnAssignStmt),
    AugAssign(AugAssignStmt),
    Expr(ExprStmt),
    Return(ReturnStmt),
    Raise(RaiseStmt),
    Pass(PassStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    If(IfStmt),
    IfLet(IfLetStmt),
    While(WhileStmt),
    For(ForStmt),
    Match(MatchStmt),
    Try(TryStmt),
    With(WithStmt),
    Unsafe(UnsafeBlock),
    Import(ImportStmt),
    FromImport(FromImportStmt),
    Function(FunctionDecl),
    Class(ClassDecl),
}
```

Nested function and class declarations remain possible because Python permits them.

## Blocks

```rust
struct Block {
    id: NodeId,
    span: Span,
    statements: Vec<Stmt>,
}
```

Indentation is syntax only; the AST stores blocks structurally.

## `if let`

```rust
struct IfLetStmt {
    id: NodeId,
    span: Span,
    pattern: Pattern,
    value: Expr,
    body: Block,
    else_body: Option<Block>,
}
```

Example:

```python
if let Some(user) = find_user(id):
    print(user.name)
else:
    ...
```

The parser does not determine whether the pattern is compatible with the value. HIR/type checking owns that decision.

## Match

```rust
struct MatchStmt {
    id: NodeId,
    span: Span,
    subject: Expr,
    arms: Vec<MatchArm>,
}

struct MatchArm {
    id: NodeId,
    span: Span,
    pattern: Pattern,
    guard: Option<Expr>,
    body: Block,
}
```

Exhaustiveness is a semantic property and is not recorded in the syntax AST.

## Unsafe blocks

```rust
struct UnsafeBlock {
    id: NodeId,
    span: Span,
    body: Block,
}
```

`unsafe` has no memory-management meaning. Semantic analysis uses lexical nesting of `UnsafeBlock` to permit otherwise-rejected dynamic operations.

## Expressions

```rust
enum Expr {
    Name(NameExpr),
    Literal(LiteralExpr),
    Tuple(TupleExpr),
    List(ListExpr),
    Dict(DictExpr),
    Set(SetExpr),
    Attribute(AttributeExpr),
    Subscript(SubscriptExpr),
    Call(CallExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Bool(BoolExpr),
    Compare(CompareExpr),
    Conditional(ConditionalExpr),
    Lambda(LambdaExpr),
    Await(AwaitExpr),
    Try(TryExpr),
    Comprehension(ComprehensionExpr),
}
```

The exact ordinary-Python expression subset follows `V0_SYNTAX_SPEC.md`.

## Postfix `?`

Postfix propagation is represented explicitly:

```rust
struct TryExpr {
    id: NodeId,
    span: Span,
    operand: Box<Expr>,
    question_span: Span,
}
```

Example:

```python
user = repository.find(id)?
```

becomes an assignment whose value is `Expr::Try` wrapping the call expression.

It must **not** be desugared by the parser. Semantic analysis must first determine whether the operand is `Result` or `Option` and whether the enclosing function permits propagation.

## Calls

```rust
struct CallExpr {
    id: NodeId,
    span: Span,
    callee: Box<Expr>,
    args: Vec<CallArg>,
}

enum CallArg {
    Positional(Expr),
    Keyword { name: Ident, value: Expr },
    Star(Expr),
    DoubleStar(Expr),
}
```

Constructors such as `Ok(...)`, `Err(...)`, and `Some(...)` are initially ordinary call syntax. HIR resolves them to Rustype intrinsic constructors when applicable.

## Type expressions

Types use a dedicated syntax tree rather than general expression nodes.

```rust
enum TypeExpr {
    Name(TypeName),
    Qualified(TypePath),
    Apply(TypeApply),
    Union(TypeUnion),
    Tuple(TypeTuple),
}
```

Examples:

```text
User                   -> Name
models.User            -> Qualified
list[User]             -> Apply
Result[User, Error]    -> Apply
User | None            -> Union
```

Using a dedicated type tree prevents arbitrary runtime expressions from becoming types accidentally.

## Patterns

```rust
enum Pattern {
    Wildcard(WildcardPattern),
    Binding(BindingPattern),
    Literal(LiteralPattern),
    Variant(VariantPattern),
    Tuple(TuplePattern),
    Or(OrPattern),
}
```

Variant patterns are first-class:

```rust
struct VariantPattern {
    id: NodeId,
    span: Span,
    path: TypePath,
    args: Vec<Pattern>,
}
```

Examples:

```text
Ok(value)
Err(error)
Some(user)
None
Move(x, y)
```

`None` is parsed as a distinguished literal/variant-compatible pattern token. Semantic analysis decides whether it means Rustype `Option::None` or Python `None` based on the matched subject type and context.

## HIR boundary

The syntax AST deliberately does not contain:

- resolved symbol IDs,
- inferred or declared semantic types,
- selected enum declarations,
- selected trait declarations,
- variant constructors,
- whether a call is intrinsic,
- `?` propagation mode,
- match exhaustiveness,
- safe/unsafe classification,
- generated temporaries.

Those belong in HIR or lowering IR.

## HIR sketch

HIR mirrors the AST structurally but replaces names with resolved symbols and attaches semantic types.

Conceptually:

```rust
struct HirExpr {
    id: HirId,
    origin: Origin,
    ty: TypeId,
    kind: HirExprKind,
}

enum HirExprKind {
    Local(SymbolId),
    Call(HirCall),
    VariantConstruct(VariantId, Vec<HirExpr>),
    Try {
        operand: Box<HirExpr>,
        mode: PropagationMode,
    },
    ...
}
```

```rust
enum PropagationMode {
    Result {
        ok: TypeId,
        err: TypeId,
    },
    Option {
        some: TypeId,
    },
}
```

HIR is the first layer where the compiler may say with certainty what `?`, `Some`, `None`, `Ok`, `Err`, or a variant pattern means.

## Stable identity and incremental work

`NodeId` values only need to be stable for one parse initially. The implementation should avoid assuming IDs equal source offsets so a future incremental parser can preserve node identity across edits.

`HirId`, `SymbolId`, `TypeId`, `EnumId`, `VariantId`, and `TraitId` should be arena/interning IDs, not pointers.

## Required parser tests

Each v0 construct requires:

- source-to-AST snapshot tests,
- span assertions,
- malformed-syntax diagnostics,
- nesting tests,
- ambiguity tests where Python and Rustype syntax overlap.

The parser test suite should never need to inspect generated Python.