#![forbid(unsafe_code)]

use rustype_ast::{
    Diagnostic, FileId, FunctionDecl, Ident, Item, Module, NewtypeDecl, NodeId, Param, Span,
    TypeExpr,
};

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
        let span = Span::new(file, start, offset + line.len());

        if trimmed.is_empty() || trimmed.starts_with('#') {
            offset += raw_line.len();
            continue;
        }

        if line.contains('\t') {
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

        parse_top_level(
            trimmed,
            file,
            start,
            span,
            &mut ids,
            &mut items,
            &mut diagnostics,
        );
        offset += raw_line.len();
    }

    ParseOutput {
        module: Module {
            id: ids.next(),
            span: Span::new(file, 0, source.len()),
            items,
        },
        diagnostics,
    }
}

fn parse_top_level(
    line: &str,
    file: FileId,
    base: usize,
    span: Span,
    ids: &mut IdGen,
    items: &mut Vec<Item>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(item) = parse_newtype(line, file, base, ids, diagnostics) {
        items.push(Item::Newtype(item));
    } else if let Some(item) = parse_function_header(line, file, base, ids, diagnostics) {
        items.push(Item::Function(item));
    } else if line.starts_with("def ") || line.starts_with("async def ") {
        diagnostics.push(Diagnostic::error(
            "RYP0002",
            "use `fn` or `async fn` in .rpy source; Python `def` is not valid Rustype v0 syntax",
            span,
        ));
    } else if !starts_known_construct(line) {
        diagnostics.push(Diagnostic::error(
            "RYP0003",
            format!("unsupported top-level syntax in bootstrap parser: `{line}`"),
            span,
        ));
    }
}

fn starts_known_construct(line: &str) -> bool {
    line.starts_with("newtype ") || line.starts_with("fn ") || line.starts_with("async fn ")
}

fn parse_newtype(
    line: &str,
    file: FileId,
    base: usize,
    ids: &mut IdGen,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<NewtypeDecl> {
    let rest = line.strip_prefix("newtype ")?;
    let Some((name_raw, ty_raw)) = rest.split_once('=') else {
        diagnostics.push(Diagnostic::error(
            "RYP0101",
            "expected `newtype Name = Type`",
            line_span(file, base, line),
        ));
        return None;
    };

    let name = name_raw.trim();
    let ty = ty_raw.trim();
    if !is_identifier(name) || ty.is_empty() {
        diagnostics.push(Diagnostic::error(
            "RYP0102",
            "invalid newtype declaration",
            line_span(file, base, line),
        ));
        return None;
    }

    let name_offset = line.find(name).unwrap_or(0);
    let ty_offset = line.rfind(ty).unwrap_or(0);

    Some(NewtypeDecl {
        id: ids.next(),
        span: line_span(file, base, line),
        name: ident(ids, file, base + name_offset, name),
        underlying: TypeExpr {
            id: ids.next(),
            span: Span::new(file, base + ty_offset, base + ty_offset + ty.len()),
            text: ty.to_owned(),
        },
    })
}

fn parse_function_header(
    line: &str,
    file: FileId,
    base: usize,
    ids: &mut IdGen,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<FunctionDecl> {
    let (is_async, rest) = function_prefix(line)?;
    let Some((signature, return_and_colon)) = rest.split_once("->") else {
        diagnostics.push(Diagnostic::error(
            "RYP0201",
            "Rustype functions require an explicit return type",
            line_span(file, base, line),
        ));
        return None;
    };
    let Some(return_raw) = return_and_colon.strip_suffix(':') else {
        diagnostics.push(Diagnostic::error(
            "RYP0202",
            "expected `:` after function signature",
            line_span(file, base, line),
        ));
        return None;
    };
    let return_ty = return_raw.trim();
    if return_ty.is_empty() {
        diagnostics.push(Diagnostic::error(
            "RYP0201",
            "Rustype functions require an explicit return type",
            line_span(file, base, line),
        ));
        return None;
    }

    let (name, params_raw) = parse_signature_shape(signature, file, base, line, diagnostics)?;
    let params = parse_params(params_raw, line, file, base, ids, diagnostics)?;
    let name_offset = line.find(name).unwrap_or(0);
    let return_offset = line.rfind(return_ty).unwrap_or(0);

    Some(FunctionDecl {
        id: ids.next(),
        span: line_span(file, base, line),
        is_async,
        name: ident(ids, file, base + name_offset, name),
        params,
        return_type: TypeExpr {
            id: ids.next(),
            span: Span::new(
                file,
                base + return_offset,
                base + return_offset + return_ty.len(),
            ),
            text: return_ty.to_owned(),
        },
    })
}

fn function_prefix(line: &str) -> Option<(bool, &str)> {
    if let Some(rest) = line.strip_prefix("async fn ") {
        Some((true, rest))
    } else {
        line.strip_prefix("fn ").map(|rest| (false, rest))
    }
}

fn parse_signature_shape<'a>(
    signature: &'a str,
    file: FileId,
    base: usize,
    line: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<(&'a str, &'a str)> {
    let Some(open) = signature.find('(') else {
        diagnostics.push(Diagnostic::error(
            "RYP0203",
            "expected `(` in function signature",
            line_span(file, base, line),
        ));
        return None;
    };
    let Some(close) = signature.rfind(')') else {
        diagnostics.push(Diagnostic::error(
            "RYP0204",
            "expected `)` in function signature",
            line_span(file, base, line),
        ));
        return None;
    };
    if close < open {
        diagnostics.push(Diagnostic::error(
            "RYP0204",
            "malformed function parameter list",
            line_span(file, base, line),
        ));
        return None;
    }

    let name = signature[..open].trim();
    if !is_identifier(name) {
        diagnostics.push(Diagnostic::error(
            "RYP0205",
            "invalid function name",
            line_span(file, base, line),
        ));
        return None;
    }

    Some((name, &signature[open + 1..close]))
}

fn parse_params(
    params_raw: &str,
    line: &str,
    file: FileId,
    base: usize,
    ids: &mut IdGen,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Vec<Param>> {
    let mut params = Vec::new();
    let mut search_from = line.find('(').map_or(0, |index| index + 1);

    for raw in params_raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let Some((param_name, annotation)) = raw.split_once(':') else {
            diagnostics.push(Diagnostic::error(
                "RYP0206",
                format!("parameter `{raw}` requires a type annotation"),
                line_span(file, base, line),
            ));
            return None;
        };
        let param_name = param_name.trim();
        let annotation = annotation.trim();
        if !is_identifier(param_name) || annotation.is_empty() {
            diagnostics.push(Diagnostic::error(
                "RYP0207",
                "invalid function parameter",
                line_span(file, base, line),
            ));
            return None;
        }

        let local = line[search_from..]
            .find(param_name)
            .map_or(search_from, |index| search_from + index);
        search_from = local + raw.len();
        params.push(Param {
            id: ids.next(),
            span: Span::new(file, base + local, base + local + raw.len()),
            name: ident(ids, file, base + local, param_name),
            annotation: TypeExpr {
                id: ids.next(),
                span: Span::new(file, base + local, base + local + raw.len()),
                text: annotation.to_owned(),
            },
        });
    }

    Some(params)
}

fn ident(ids: &mut IdGen, file: FileId, start: usize, name: &str) -> Ident {
    Ident {
        id: ids.next(),
        span: Span::new(file, start, start + name.len()),
        name: name.to_owned(),
    }
}

fn line_span(file: FileId, base: usize, line: &str) -> Span {
    Span::new(file, base, base + line.len())
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
