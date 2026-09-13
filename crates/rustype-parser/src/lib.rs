#![forbid(unsafe_code)]

use rustype_ast::{Diagnostic, FileId, FunctionDecl, Ident, Item, Module, NewtypeDecl, NodeId, Param, Span, TypeExpr};

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
    let mut ids = IdGen::default();
    let mut items = Vec::new();
    let mut diagnostics = Vec::new();
    let mut offset = 0usize;

    for raw_line in source.split_inclusive('\n') {
        let line = raw_line.trim_end_matches(['\r', '\n']);
        let trimmed = line.trim();
        let start = offset + line.len().saturating_sub(line.trim_start().len());
        let end = offset + line.len();
        let span = Span::new(file, start as u32, end as u32);

        if trimmed.is_empty() || trimmed.starts_with('#') {
            offset += raw_line.len();
            continue;
        }

        if line.starts_with('\t') || line.contains("\t") {
            diagnostics.push(Diagnostic::error(
                "RYP0001",
                "tabs are not allowed in Rustype v0 source",
                span,
            ));
            offset += raw_line.len();
            continue;
        }

        if line.len() != line.trim_start().len() {
            // Body parsing is intentionally deferred from the bootstrap parser. Indented
            // lines are accepted as opaque body text for now.
            offset += raw_line.len();
            continue;
        }

        if let Some(item) = parse_newtype(trimmed, file, start as u32, &mut ids, &mut diagnostics) {
            items.push(Item::Newtype(item));
        } else if let Some(item) = parse_function_header(trimmed, file, start as u32, &mut ids, &mut diagnostics) {
            items.push(Item::Function(item));
        } else if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            diagnostics.push(Diagnostic::error(
                "RYP0002",
                "use `fn` or `async fn` in .rpy source; Python `def` is not valid Rustype v0 syntax",
                span,
            ));
        } else if trimmed.starts_with("newtype ") || trimmed.starts_with("fn ") || trimmed.starts_with("async fn ") {
            // A construct prefix was recognized but parsing failed; a targeted diagnostic
            // has already been emitted by the parser helper.
        } else {
            diagnostics.push(Diagnostic::error(
                "RYP0003",
                format!("unsupported top-level syntax in bootstrap parser: `{trimmed}`"),
                span,
            ));
        }

        offset += raw_line.len();
    }

    let module_span = Span::new(file, 0, source.len() as u32);
    ParseOutput {
        module: Module {
            id: ids.next(),
            span: module_span,
            items,
        },
        diagnostics,
    }
}

fn parse_newtype(
    line: &str,
    file: FileId,
    base: u32,
    ids: &mut IdGen,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<NewtypeDecl> {
    let rest = line.strip_prefix("newtype ")?;
    let Some((name_raw, ty_raw)) = rest.split_once('=') else {
        diagnostics.push(Diagnostic::error(
            "RYP0101",
            "expected `newtype Name = Type`",
            Span::new(file, base, base + line.len() as u32),
        ));
        return None;
    };

    let name = name_raw.trim();
    let ty = ty_raw.trim();
    if !is_identifier(name) || ty.is_empty() {
        diagnostics.push(Diagnostic::error(
            "RYP0102",
            "invalid newtype declaration",
            Span::new(file, base, base + line.len() as u32),
        ));
        return None;
    }

    let name_offset = line.find(name).unwrap_or(0) as u32;
    let ty_offset = line.rfind(ty).unwrap_or(0) as u32;

    Some(NewtypeDecl {
        id: ids.next(),
        span: Span::new(file, base, base + line.len() as u32),
        name: Ident {
            id: ids.next(),
            span: Span::new(file, base + name_offset, base + name_offset + name.len() as u32),
            name: name.to_owned(),
        },
        underlying: TypeExpr {
            id: ids.next(),
            span: Span::new(file, base + ty_offset, base + ty_offset + ty.len() as u32),
            text: ty.to_owned(),
        },
    })
}

fn parse_function_header(
    line: &str,
    file: FileId,
    base: u32,
    ids: &mut IdGen,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<FunctionDecl> {
    let (is_async, rest) = if let Some(rest) = line.strip_prefix("async fn ") {
        (true, rest)
    } else if let Some(rest) = line.strip_prefix("fn ") {
        (false, rest)
    } else {
        return None;
    };

    let Some((signature, return_and_colon)) = rest.split_once("->") else {
        diagnostics.push(Diagnostic::error(
            "RYP0201",
            "Rustype functions require an explicit return type",
            Span::new(file, base, base + line.len() as u32),
        ));
        return None;
    };
    let Some(return_raw) = return_and_colon.strip_suffix(':') else {
        diagnostics.push(Diagnostic::error(
            "RYP0202",
            "expected `:` after function signature",
            Span::new(file, base, base + line.len() as u32),
        ));
        return None;
    };
    let return_ty = return_raw.trim();

    let Some(open) = signature.find('(') else {
        diagnostics.push(Diagnostic::error("RYP0203", "expected `(` in function signature", Span::new(file, base, base + line.len() as u32)));
        return None;
    };
    let Some(close) = signature.rfind(')') else {
        diagnostics.push(Diagnostic::error("RYP0204", "expected `)` in function signature", Span::new(file, base, base + line.len() as u32)));
        return None;
    };
    if close < open {
        return None;
    }

    let name = signature[..open].trim();
    if !is_identifier(name) {
        diagnostics.push(Diagnostic::error("RYP0205", "invalid function name", Span::new(file, base, base + line.len() as u32)));
        return None;
    }

    let params_raw = &signature[open + 1..close];
    let mut params = Vec::new();
    for raw in params_raw.split(',').map(str::trim).filter(|value| !value.is_empty()) {
        let Some((param_name, annotation)) = raw.split_once(':') else {
            diagnostics.push(Diagnostic::error(
                "RYP0206",
                format!("parameter `{raw}` requires a type annotation"),
                Span::new(file, base, base + line.len() as u32),
            ));
            return None;
        };
        let param_name = param_name.trim();
        let annotation = annotation.trim();
        if !is_identifier(param_name) || annotation.is_empty() {
            diagnostics.push(Diagnostic::error("RYP0207", "invalid function parameter", Span::new(file, base, base + line.len() as u32)));
            return None;
        }
        let local = line.find(param_name).unwrap_or(0) as u32;
        params.push(Param {
            id: ids.next(),
            span: Span::new(file, base + local, base + local + raw.len() as u32),
            name: Ident {
                id: ids.next(),
                span: Span::new(file, base + local, base + local + param_name.len() as u32),
                name: param_name.to_owned(),
            },
            annotation: TypeExpr {
                id: ids.next(),
                span: Span::new(file, base + local, base + local + raw.len() as u32),
                text: annotation.to_owned(),
            },
        });
    }

    let name_offset = line.find(name).unwrap_or(0) as u32;
    let return_offset = line.rfind(return_ty).unwrap_or(0) as u32;
    Some(FunctionDecl {
        id: ids.next(),
        span: Span::new(file, base, base + line.len() as u32),
        is_async,
        name: Ident {
            id: ids.next(),
            span: Span::new(file, base + name_offset, base + name_offset + name.len() as u32),
            name: name.to_owned(),
        },
        params,
        return_type: TypeExpr {
            id: ids.next(),
            span: Span::new(file, base + return_offset, base + return_offset + return_ty.len() as u32),
            text: return_ty.to_owned(),
        },
    })
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_alphabetic()) && chars.all(|ch| ch == '_' || ch.is_alphanumeric())
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
    fn rejects_python_def() {
        let output = parse_module(FileId(0), "def greet(name: str) -> str:\n    return name\n");
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(output.diagnostics[0].code, "RYP0002");
    }

    #[test]
    fn rejects_tabs() {
        let output = parse_module(FileId(0), "fn greet(name: str) -> str:\n\treturn name\n");
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(output.diagnostics[0].code, "RYP0001");
    }
}
