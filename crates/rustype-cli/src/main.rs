#![forbid(unsafe_code)]

use std::{env, fs, path::PathBuf, process::ExitCode};

use rustype_ast::FileId;
use rustype_parser::parse_module;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(usage());
    };

    if command != "check" {
        return Err(format!("unknown command `{command}`\n\n{}", usage()));
    }

    let Some(path) = args.next() else {
        return Err(format!("missing .rpy file\n\n{}", usage()));
    };
    if args.next().is_some() {
        return Err(format!("too many arguments\n\n{}", usage()));
    }

    let path = PathBuf::from(path);
    if path.extension().and_then(|value| value.to_str()) != Some("rpy") {
        return Err(format!("expected a .rpy file: {}", path.display()));
    }

    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let output = parse_module(FileId(0), &source);

    if output.diagnostics.is_empty() {
        println!(
            "checked {}: {} top-level Rustype item(s)",
            path.display(),
            output.module.items.len()
        );
        return Ok(());
    }

    let line_index = LineIndex::new(&source);
    for diagnostic in &output.diagnostics {
        let (line, column) = line_index.line_column(diagnostic.span.start);
        eprintln!(
            "{}:{}:{}: error[{}]: {}",
            path.display(),
            line,
            column,
            diagnostic.code,
            diagnostic.message
        );
    }

    Err(format!(
        "{} error(s) found in {}",
        output.diagnostics.len(),
        path.display()
    ))
}

fn usage() -> String {
    "usage: rustype check <file.rpy>".to_owned()
}

struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    fn new(source: &str) -> Self {
        let mut starts = vec![0];
        for (index, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(index + 1);
            }
        }
        Self { starts }
    }

    fn line_column(&self, offset: usize) -> (usize, usize) {
        let line_index = self
            .starts
            .partition_point(|start| *start <= offset)
            .saturating_sub(1);
        let line_start = self.starts[line_index];
        (line_index + 1, offset.saturating_sub(line_start) + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::LineIndex;

    #[test]
    fn maps_offsets_to_one_based_locations() {
        let index = LineIndex::new("one\ntwo\n");
        assert_eq!(index.line_column(0), (1, 1));
        assert_eq!(index.line_column(4), (2, 1));
    }
}
