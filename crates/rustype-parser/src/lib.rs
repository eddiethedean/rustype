#![forbid(unsafe_code)]

use rustype_ast::{
    Diagnostic, FileId, FunctionDecl, Ident, Item, Module, NewtypeDecl, NodeId, Param, Span,
    TypeExpr,
};
use rustype_lexer::{lex, Token, TokenKind};

#[derive(Debug, Default)]
struct IdGen(u32);

impl IdGen {
    fn next(&mut self) -> NodeId {
        let id = NodeId(self.0);
        self.0 += 1;
        id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutput {
    pub module: Module,
    pub diagnostics: Vec<Diagnostic>,
}

#[must_use]
pub fn parse_module(file: FileId, source: &str) -> ParseOutput {
    let lexed = lex(file, source);
    let mut parser = Parser::new(file, source, lexed.tokens, lexed.diagnostics);
    parser.parse_module()
}

struct Parser<'source> {
    file: FileId,
    source: &'source str,
    tokens: Vec<Token>,
    cursor: usize,
    ids: IdGen,
    diagnostics: Vec<Diagnostic>,
}

impl<'source> Parser<'source> {
    fn new(
        file: FileId,
        source: &'source str,
        tokens: Vec<Token>,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        Self {
            file,
            source,
            tokens,
            cursor: 0,
            ids: IdGen::default(),
            diagnostics,
        }
    }

    fn parse_module(&mut self) -> ParseOutput {
        let mut items = Vec::new();
        while !self.at(TokenKind::Eof) {
            if self.take(TokenKind::Newline) || self.take(TokenKind::Dedent) {
                continue;
            }
            match self.current_kind() {
                TokenKind::Newtype => {
                    if let Some(item) = self.parse_newtype() {
                        items.push(Item::Newtype(item));
                    }
                }
                TokenKind::Fn | TokenKind::Async => {
                    if let Some(item) = self.parse_function() {
                        items.push(Item::Function(item));
                    }
                }
                TokenKind::Def => {
                    let span = self.current().span;
                    self.diagnostics.push(Diagnostic::error(
                        "RYP0002",
                        "use `fn` or `async fn` in .rpy source; Python `def` is not valid Rustype v0 syntax",
                        span,
                    ));
                    self.skip_line();
                }
                _ => {
                    let span = self.current().span;
                    self.diagnostics.push(Diagnostic::error(
                        "RYP0003",
                        format!(
                            "unsupported top-level syntax in bootstrap parser: `{}`",
                            self.text(span)
                        ),
                        span,
                    ));
                    self.skip_line();
                }
            }
        }

        ParseOutput {
            module: Module {
                id: self.ids.next(),
                span: Span::new(self.file, 0, self.source.len()),
                items,
            },
            diagnostics: std::mem::take(&mut self.diagnostics),
        }
    }

    fn parse_newtype(&mut self) -> Option<NewtypeDecl> {
        let start = self.expect(TokenKind::Newtype, "RYP0101", "expected `newtype`")?.span.start;
        let name_token = self.expect(TokenKind::Name, "RYP0102", "expected newtype name")?;
        self.expect(TokenKind::Assign, "RYP0101", "expected `=` in newtype declaration")?;
        let type_start = self.cursor;
        while !self.at(TokenKind::Newline) && !self.at(TokenKind::Eof) {
            self.bump();
        }
        if type_start == self.cursor {
            self.error_here("RYP0102", "expected underlying type");
            self.take(TokenKind::Newline);
            return None;
        }
        let type_span = self.span_for_range(type_start, self.cursor);
        let end = type_span.end;
        self.take(TokenKind::Newline);

        Some(NewtypeDecl {
            id: self.ids.next(),
            span: Span::new(self.file, start, end),
            name: self.ident_from(name_token),
            underlying: self.type_expr(type_span),
        })
    }

    fn parse_function(&mut self) -> Option<FunctionDecl> {
        let start = self.current().span.start;
        let is_async = if self.take(TokenKind::Async) {
            if !self.take(TokenKind::Fn) {
                self.error_here("RYP0200", "`async` must be followed by `fn` in Rustype v0");
                self.skip_line();
                return None;
            }
            true
        } else {
            self.expect(TokenKind::Fn, "RYP0200", "expected `fn`")?;
            false
        };

        let name_token = self.expect(TokenKind::Name, "RYP0205", "expected function name")?;
        self.expect(TokenKind::LParen, "RYP0203", "expected `(` in function signature")?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen, "RYP0204", "expected `)` in function signature")?;
        self.expect(TokenKind::Arrow, "RYP0201", "Rustype functions require an explicit return type")?;

        let return_start = self.cursor;
        while !self.at(TokenKind::Colon) && !self.at(TokenKind::Newline) && !self.at(TokenKind::Eof) {
            self.bump();
        }
        if return_start == self.cursor {
            self.error_here("RYP0201", "Rustype functions require an explicit return type");
            self.skip_line();
            return None;
        }
        let return_span = self.span_for_range(return_start, self.cursor);
        let colon = self.expect(TokenKind::Colon, "RYP0202", "expected `:` after function signature")?;
        let end = colon.span.end;
        self.take(TokenKind::Newline);
        self.skip_suite();

        Some(FunctionDecl {
            id: self.ids.next(),
            span: Span::new(self.file, start, end),
            is_async,
            name: self.ident_from(name_token),
            params,
            return_type: self.type_expr(return_span),
        })
    }

    fn parse_params(&mut self) -> Option<Vec<Param>> {
        let mut params = Vec::new();
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            if self.take(TokenKind::Comma) {
                continue;
            }
            let name_token = self.expect(TokenKind::Name, "RYP0207", "expected parameter name")?;
            self.expect(TokenKind::Colon, "RYP0206", "function parameters require type annotations")?;
            let type_start = self.cursor;
            let mut nested = 0usize;
            while !self.at(TokenKind::Eof) {
                match self.current_kind() {
                    TokenKind::LBracket | TokenKind::LParen => {
                        nested += 1;
                        self.bump();
                    }
                    TokenKind::RBracket if nested > 0 => {
                        nested -= 1;
                        self.bump();
                    }
                    TokenKind::RParen if nested > 0 => {
                        nested -= 1;
                        self.bump();
                    }
                    TokenKind::Comma | TokenKind::RParen if nested == 0 => break,
                    _ => self.bump(),
                }
            }
            if type_start == self.cursor {
                self.error_here("RYP0207", "invalid function parameter");
                return None;
            }
            let annotation_span = self.span_for_range(type_start, self.cursor);
            let param_span = Span::new(self.file, name_token.span.start, annotation_span.end);
            params.push(Param {
                id: self.ids.next(),
                span: param_span,
                name: self.ident_from(name_token),
                annotation: self.type_expr(annotation_span),
            });
            self.take(TokenKind::Comma);
        }
        Some(params)
    }

    fn skip_suite(&mut self) {
        if !self.take(TokenKind::Indent) {
            return;
        }
        let mut depth = 1usize;
        while depth > 0 && !self.at(TokenKind::Eof) {
            match self.current_kind() {
                TokenKind::Indent => depth += 1,
                TokenKind::Dedent => depth -= 1,
                _ => {}
            }
            self.bump();
        }
    }

    fn skip_line(&mut self) {
        while !self.at(TokenKind::Newline) && !self.at(TokenKind::Eof) {
            self.bump();
        }
        self.take(TokenKind::Newline);
    }

    fn expect(&mut self, kind: TokenKind, code: &'static str, message: &'static str) -> Option<Token> {
        if self.at(kind) {
            Some(self.bump())
        } else {
            self.error_here(code, message);
            None
        }
    }

    fn error_here(&mut self, code: &'static str, message: &'static str) {
        self.diagnostics
            .push(Diagnostic::error(code, message, self.current().span));
    }

    fn type_expr(&mut self, span: Span) -> TypeExpr {
        TypeExpr {
            id: self.ids.next(),
            span,
            text: self.text(span).to_owned(),
        }
    }

    fn ident_from(&mut self, token: Token) -> Ident {
        Ident {
            id: self.ids.next(),
            span: token.span,
            name: self.text(token.span).to_owned(),
        }
    }

    fn span_for_range(&self, start: usize, end: usize) -> Span {
        let first = self.tokens[start].span;
        let last = self.tokens[end - 1].span;
        Span::new(self.file, first.start, last.end)
    }

    fn text(&self, span: Span) -> &str {
        &self.source[span.start..span.end]
    }

    fn current(&self) -> Token {
        self.tokens[self.cursor]
    }

    fn current_kind(&self) -> TokenKind {
        self.current().kind
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.current_kind() == kind
    }

    fn take(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn bump(&mut self) -> Token {
        let token = self.current();
        if token.kind != TokenKind::Eof {
            self.cursor += 1;
        }
        token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_first_vertical_slice() {
        let source = "newtype UserId = int\n\nfn greet(id: UserId, name: str) -> str:\n    return f\"{id}: {name}\"\n";
        let output = parse_module(FileId(0), source);
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        assert_eq!(output.module.items.len(), 2);
        assert!(matches!(output.module.items[0], Item::Newtype(_)));
        assert!(matches!(output.module.items[1], Item::Function(_)));
    }

    #[test]
    fn parses_generic_parameter_types() {
        let source = "fn parse(value: Result[list[int], str]) -> Option[int]:\n    return None\n";
        let output = parse_module(FileId(0), source);
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let Item::Function(function) = &output.module.items[0] else {
            panic!("expected function");
        };
        assert_eq!(function.params[0].annotation.text, "Result[list[int], str]");
        assert_eq!(function.return_type.text, "Option[int]");
    }

    #[test]
    fn rejects_python_def() {
        let output = parse_module(FileId(0), "def greet(name: str) -> str:\n    return name\n");
        assert!(output.diagnostics.iter().any(|diagnostic| diagnostic.code == "RYP0002"));
    }

    #[test]
    fn forwards_lexer_diagnostics() {
        let output = parse_module(FileId(0), "fn greet(name: str) -> str:\n\treturn name\n");
        assert!(output.diagnostics.iter().any(|diagnostic| diagnostic.code == "RYL0001"));
    }

    #[test]
    fn tracks_repeated_parameter_names_by_position() {
        let source = "fn same(same: str, other: str) -> str:\n    return other\n";
        let output = parse_module(FileId(0), source);
        let Item::Function(function) = &output.module.items[0] else {
            panic!("expected function");
        };
        assert!(function.params[0].name.span.start > function.name.span.start);
    }
}
