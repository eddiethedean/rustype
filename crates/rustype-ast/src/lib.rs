#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: FileId,
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[must_use]
    pub const fn new(file: FileId, start: usize, end: usize) -> Self {
        Self { file, start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeExpr {
    pub id: NodeId,
    pub span: Span,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewtypeDecl {
    pub id: NodeId,
    pub span: Span,
    pub name: Ident,
    pub underlying: TypeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub id: NodeId,
    pub span: Span,
    pub name: Ident,
    pub annotation: TypeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDecl {
    pub id: NodeId,
    pub span: Span,
    pub is_async: bool,
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: TypeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Newtype(NewtypeDecl),
    Function(FunctionDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub id: NodeId,
    pub span: Span,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

impl Diagnostic {
    #[must_use]
    pub fn error(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }
}
