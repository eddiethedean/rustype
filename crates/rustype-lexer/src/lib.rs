#![forbid(unsafe_code)]

use rustype_ast::{Diagnostic, FileId, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Name,
    Number,
    String,
    Newline,
    Indent,
    Dedent,
    Eof,
    Fn,
    Def,
    Async,
    Newtype,
    Enum,
    Trait,
    Class,
    Unsafe,
    If,
    Else,
    Elif,
    Let,
    Match,
    Case,
    Return,
    Raise,
    Pass,
    Break,
    Continue,
    While,
    For,
    In,
    Try,
    Except,
    Finally,
    With,
    As,
    Import,
    From,
    Await,
    True,
    False,
    None,
    And,
    Or,
    Not,
    Plus,
    Minus,
    Star,
    DoubleStar,
    Slash,
    DoubleSlash,
    Percent,
    At,
    Pipe,
    Ampersand,
    Caret,
    Tilde,
    LeftShift,
    RightShift,
    Walrus,
    Assign,
    EqEq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Arrow,
    Question,
    Colon,
    Comma,
    Dot,
    Semicolon,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    #[must_use]
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

#[must_use]
pub fn lex(file: FileId, source: &str) -> LexOutput {
    Lexer::new(file, source).run()
}

struct Lexer<'source> {
    file: FileId,
    source: &'source str,
    pos: usize,
    at_line_start: bool,
    nesting: usize,
    indents: Vec<usize>,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl<'source> Lexer<'source> {
    fn new(file: FileId, source: &'source str) -> Self {
        Self {
            file,
            source,
            pos: 0,
            at_line_start: true,
            nesting: 0,
            indents: vec![0],
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn run(mut self) -> LexOutput {
        while self.pos < self.source.len() {
            if self.at_line_start && self.nesting == 0 {
                self.lex_indentation();
                if self.pos >= self.source.len() {
                    break;
                }
            }

            let Some(ch) = self.peek_char() else {
                break;
            };
            match ch {
                ' ' | '\x0c' => {
                    self.bump_char();
                }
                '\t' => self.lex_tab(),
                '#' => self.skip_comment(),
                '\n' | '\r' => self.lex_newline(),
                '0'..='9' => self.lex_number(),
                '\'' | '"' => self.lex_string(),
                _ if is_identifier_start(ch) => self.lex_name(),
                _ => self.lex_operator(),
            }
        }

        let eof = self.source.len();
        while self.indents.len() > 1 {
            self.indents.pop();
            self.push(TokenKind::Dedent, eof, eof);
        }
        self.push(TokenKind::Eof, eof, eof);

        LexOutput {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn lex_indentation(&mut self) {
        let start = self.pos;
        let mut width = 0usize;
        while let Some(ch) = self.peek_char() {
            match ch {
                ' ' => {
                    width += 1;
                    self.bump_char();
                }
                '\x0c' => {
                    self.bump_char();
                }
                '\t' => {
                    let tab_start = self.pos;
                    self.bump_char();
                    self.diagnostics.push(Diagnostic::error(
                        "RYL0001",
                        "tabs are not allowed in Rustype v0 source",
                        Span::new(self.file, tab_start, self.pos),
                    ));
                    width += 1;
                }
                _ => break,
            }
        }

        if matches!(self.peek_char(), Some('\n' | '\r' | '#') | None) {
            self.at_line_start = false;
            return;
        }

        let current = self.indents.last().copied().unwrap_or(0);
        if width > current {
            self.indents.push(width);
            self.push(TokenKind::Indent, start, self.pos);
        } else if width < current {
            while self.indents.last().is_some_and(|indent| *indent > width) {
                self.indents.pop();
                self.push(TokenKind::Dedent, start, self.pos);
            }
            if self.indents.last().copied().unwrap_or(0) != width {
                self.diagnostics.push(Diagnostic::error(
                    "RYL0002",
                    "inconsistent indentation",
                    Span::new(self.file, start, self.pos),
                ));
            }
        }
        self.at_line_start = false;
    }

    fn lex_tab(&mut self) {
        let start = self.pos;
        self.bump_char();
        self.diagnostics.push(Diagnostic::error(
            "RYL0001",
            "tabs are not allowed in Rustype v0 source",
            Span::new(self.file, start, self.pos),
        ));
    }

    fn lex_newline(&mut self) {
        let start = self.pos;
        if self.peek_char() == Some('\r') {
            self.bump_char();
            if self.peek_char() == Some('\n') {
                self.bump_char();
            }
        } else {
            self.bump_char();
        }
        if self.nesting == 0 {
            self.push(TokenKind::Newline, start, self.pos);
        }
        self.at_line_start = true;
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.peek_char() {
            if matches!(ch, '\n' | '\r') {
                break;
            }
            self.bump_char();
        }
    }

    fn lex_name(&mut self) {
        let start = self.pos;
        self.bump_char();
        while self.peek_char().is_some_and(is_identifier_continue) {
            self.bump_char();
        }
        let text = &self.source[start..self.pos];
        self.push(
            keyword_kind(text).unwrap_or(TokenKind::Name),
            start,
            self.pos,
        );
    }

    fn lex_number(&mut self) {
        let start = self.pos;
        self.bump_char();
        while self
            .peek_char()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.'))
        {
            self.bump_char();
        }
        self.push(TokenKind::Number, start, self.pos);
    }

    fn lex_string(&mut self) {
        let start = self.pos;
        let quote = self.peek_char().unwrap_or('"');
        self.bump_char();
        let triple = self.peek_char() == Some(quote) && self.peek_nth_char(1) == Some(quote);
        if triple {
            self.bump_char();
            self.bump_char();
        }

        let mut terminated = false;
        while let Some(ch) = self.peek_char() {
            if ch == '\\' {
                self.bump_char();
                if self.peek_char().is_some() {
                    self.bump_char();
                }
                continue;
            }
            if ch == quote {
                self.bump_char();
                if triple {
                    if self.peek_char() == Some(quote) && self.peek_nth_char(1) == Some(quote) {
                        self.bump_char();
                        self.bump_char();
                        terminated = true;
                        break;
                    }
                    continue;
                }
                terminated = true;
                break;
            }
            if !triple && matches!(ch, '\n' | '\r') {
                break;
            }
            self.bump_char();
        }

        if terminated {
            self.push(TokenKind::String, start, self.pos);
        } else {
            self.diagnostics.push(Diagnostic::error(
                "RYL0003",
                "unterminated string literal",
                Span::new(self.file, start, self.pos),
            ));
        }
    }

    fn lex_operator(&mut self) {
        let start = self.pos;
        let Some(ch) = self.bump_char() else {
            return;
        };
        let kind = match ch {
            '+' => TokenKind::Plus,
            '-' if self.take_if('>') => TokenKind::Arrow,
            '-' => TokenKind::Minus,
            '*' if self.take_if('*') => TokenKind::DoubleStar,
            '*' => TokenKind::Star,
            '/' if self.take_if('/') => TokenKind::DoubleSlash,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '@' => TokenKind::At,
            '|' => TokenKind::Pipe,
            '&' => TokenKind::Ampersand,
            '^' => TokenKind::Caret,
            '~' => TokenKind::Tilde,
            '<' if self.take_if('<') => TokenKind::LeftShift,
            '<' if self.take_if('=') => TokenKind::LessEq,
            '<' => TokenKind::Less,
            '>' if self.take_if('>') => TokenKind::RightShift,
            '>' if self.take_if('=') => TokenKind::GreaterEq,
            '>' => TokenKind::Greater,
            ':' if self.take_if('=') => TokenKind::Walrus,
            ':' => TokenKind::Colon,
            '=' if self.take_if('=') => TokenKind::EqEq,
            '=' => TokenKind::Assign,
            '!' if self.take_if('=') => TokenKind::NotEq,
            '?' => TokenKind::Question,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            ';' => TokenKind::Semicolon,
            '(' => {
                self.nesting += 1;
                TokenKind::LParen
            }
            ')' => {
                self.nesting = self.nesting.saturating_sub(1);
                TokenKind::RParen
            }
            '[' => {
                self.nesting += 1;
                TokenKind::LBracket
            }
            ']' => {
                self.nesting = self.nesting.saturating_sub(1);
                TokenKind::RBracket
            }
            '{' => {
                self.nesting += 1;
                TokenKind::LBrace
            }
            '}' => {
                self.nesting = self.nesting.saturating_sub(1);
                TokenKind::RBrace
            }
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "RYL0004",
                    format!("unexpected character `{ch}`"),
                    Span::new(self.file, start, self.pos),
                ));
                return;
            }
        };
        self.push(kind, start, self.pos);
    }

    fn take_if(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.bump_char();
            true
        } else {
            false
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_nth_char(&self, n: usize) -> Option<char> {
        self.source[self.pos..].chars().nth(n)
    }

    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens
            .push(Token::new(kind, Span::new(self.file, start, end)));
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}

fn keyword_kind(text: &str) -> Option<TokenKind> {
    Some(match text {
        "fn" => TokenKind::Fn,
        "def" => TokenKind::Def,
        "async" => TokenKind::Async,
        "newtype" => TokenKind::Newtype,
        "enum" => TokenKind::Enum,
        "trait" => TokenKind::Trait,
        "class" => TokenKind::Class,
        "unsafe" => TokenKind::Unsafe,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "elif" => TokenKind::Elif,
        "let" => TokenKind::Let,
        "match" => TokenKind::Match,
        "case" => TokenKind::Case,
        "return" => TokenKind::Return,
        "raise" => TokenKind::Raise,
        "pass" => TokenKind::Pass,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "while" => TokenKind::While,
        "for" => TokenKind::For,
        "in" => TokenKind::In,
        "try" => TokenKind::Try,
        "except" => TokenKind::Except,
        "finally" => TokenKind::Finally,
        "with" => TokenKind::With,
        "as" => TokenKind::As,
        "import" => TokenKind::Import,
        "from" => TokenKind::From,
        "await" => TokenKind::Await,
        "True" => TokenKind::True,
        "False" => TokenKind::False,
        "None" => TokenKind::None,
        "and" => TokenKind::And,
        "or" => TokenKind::Or,
        "not" => TokenKind::Not,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        lex(FileId(0), source)
            .tokens
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    #[test]
    fn emits_indentation_tokens() {
        assert_eq!(
            kinds("fn greet() -> str:\n    return \"hi\"\npass\n"),
            vec![
                TokenKind::Fn,
                TokenKind::Name,
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::Arrow,
                TokenKind::Name,
                TokenKind::Colon,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::Return,
                TokenKind::String,
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Pass,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn recognizes_rustype_operators() {
        let output = lex(FileId(0), "value?\n");
        assert!(output.diagnostics.is_empty());
        assert!(output
            .tokens
            .iter()
            .any(|token| token.kind == TokenKind::Question));
    }

    #[test]
    fn suppresses_newlines_inside_delimiters() {
        assert_eq!(
            kinds("fn f(\n    value: int,\n) -> int:\n    return value\n"),
            vec![
                TokenKind::Fn,
                TokenKind::Name,
                TokenKind::LParen,
                TokenKind::Name,
                TokenKind::Colon,
                TokenKind::Name,
                TokenKind::Comma,
                TokenKind::RParen,
                TokenKind::Arrow,
                TokenKind::Name,
                TokenKind::Colon,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::Return,
                TokenKind::Name,
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn rejects_tabs() {
        let output = lex(FileId(0), "fn f() -> int:\n\treturn 1\n");
        assert!(output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "RYL0001"));
    }

    #[test]
    fn rejects_unterminated_strings() {
        let output = lex(FileId(0), "value = \"oops\n");
        assert!(output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "RYL0003"));
    }
}
