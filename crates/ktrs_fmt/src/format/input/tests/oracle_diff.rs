//! Differential test against ktfmt's real `KotlinInput`. Dump the oracle with
//! `tools/ktfmt-oracle/engine/engine-oracle.sh tokens <src> <out>` (one `<file>.tokens` per
//! source), then `KTRS_INPUT_ORACLE=<out> KTRS_INPUT_CORPUS=<src> cargo test -p ktrs_fmt
//! input_oracle -- --ignored`.

use std::fmt::Write;
use std::path::{Path, PathBuf};

use ktrs_parser::FileKind;

use crate::doc::{Input, Tok};
use crate::format::input::KotlinInput;

fn esc(s: &str) -> String {
    let mut b = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => b.push_str("\\\\"),
            '"' => b.push_str("\\\""),
            '\n' => b.push_str("\\n"),
            '\r' => b.push_str("\\r"),
            '\t' => b.push_str("\\t"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => write!(b, "\\u{:04x}", c as u32).unwrap(),
            c => b.push(c),
        }
    }
    b.push('"');
    b
}

/// The same text `InputDump.dump` prints, with UTF-16 positions.
pub(super) fn dump(code: &str) -> String {
    let parse = ktrs_parser::parse_file(code, FileKind::Script);
    if parse.has_errors() {
        return "ERROR ParseError".to_string();
    }
    let input = match KotlinInput::new(code, &parse.syntax()) {
        Ok(input) => input,
        Err(e) => return format!("ERROR ParseError {e}\n"),
    };
    let mut utf16_at = vec![0i32; code.len() + 1];
    let mut u = 0;
    for (i, c) in code.char_indices() {
        utf16_at[i] = u;
        u += c.len_utf16() as i32;
    }
    utf16_at[code.len()] = u;
    let tok = |t: &dyn Tok| {
        let mut s = format!(
            "{}@{}:{}",
            t.get_index(),
            utf16_at[t.get_position() as usize],
            esc(t.get_text())
        );
        if t.get_text() != t.get_original_text() {
            write!(s, "/{}", esc(t.get_original_text())).unwrap();
        }
        s
    };
    let mut out = format!("kN {}\n", input.get_kn());
    for t in input.get_tokens() {
        out.push('B');
        for x in t.get_toks_before() {
            write!(out, " {}", tok(&**x)).unwrap();
        }
        write!(out, "\nT {}\nA", tok(&**t.get_tok())).unwrap();
        for x in t.get_toks_after() {
            write!(out, " {}", tok(&**x)).unwrap();
        }
        out.push('\n');
    }
    let io = input.input_output();
    writeln!(out, "lines {}", io.get_line_count()).unwrap();
    for i in 0..=io.get_line_count() {
        let r = io.get_ranges(i);
        writeln!(out, "R {} {}", r.lower_endpoint(), r.upper_endpoint()).unwrap();
    }
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|e| e == "tokens") {
            out.push(p);
        }
    }
}

#[test]
#[ignore = "needs KTRS_INPUT_ORACLE/KTRS_INPUT_CORPUS dumps from the JVM harness"]
fn input_oracle() {
    let oracle = PathBuf::from(std::env::var("KTRS_INPUT_ORACLE").expect("KTRS_INPUT_ORACLE"));
    let corpus = PathBuf::from(std::env::var("KTRS_INPUT_CORPUS").expect("KTRS_INPUT_CORPUS"));
    let mut files = Vec::new();
    walk(&oracle, &mut files);
    let mut failures = Vec::new();
    for expected_path in &files {
        let rel = expected_path
            .strip_prefix(&oracle)
            .unwrap()
            .with_extension("");
        let code = std::fs::read_to_string(corpus.join(&rel))
            .unwrap()
            .replace("\r\n", "\n")
            .replace('\r', "\n");
        let expected = std::fs::read_to_string(expected_path).unwrap();
        let actual = dump(&code);
        let same =
            if expected.starts_with("ERROR ParseError") && actual.starts_with("ERROR ParseError") {
                !expected.contains("Unclosed comment") || expected == actual
            } else {
                expected == actual
            };
        if !same {
            let line = expected
                .lines()
                .zip(actual.lines())
                .position(|(e, a)| e != a)
                .unwrap_or(0);
            failures.push(format!(
                "{}: first diff at line {line}\n  expected: {}\n  actual:   {}",
                rel.display(),
                expected.lines().nth(line).unwrap_or(""),
                actual.lines().nth(line).unwrap_or("")
            ));
        }
    }
    println!("{} files, {} mismatches", files.len(), failures.len());
    for f in failures.iter().take(20) {
        println!("{f}");
    }
    assert!(failures.is_empty());
}
