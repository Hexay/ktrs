//! Differential check against the JVM lexers. `tools/LexDump.java` (compiled against the jars that
//! `tools/psi-dump/psi-dump.sh` fetches) writes `<file>.<mode>.tok` next to every input file:
//! `java -cp <build;jars> LexDump kotlin|kdoc <dir> .hexlines .kt .kts`, then
//! `KTRS_ORACLE_DIR=<dir> cargo test --release -p ktrs-lexer --test oracle -- --ignored --nocapture`.
//! A `.hexlines` file batches many small cases (one hex-encoded UTF-8 text per line).

mod common;

use std::path::Path;

use common::{spans, walk};
use ktrs_lexer::Token;

/// `kind start end` per token, offsets in UTF-16 units like the JVM lexer.
fn dump(text: &str, tokens: &[Token], out: &mut Vec<String>) {
    let mut offset = 0;
    for (kind, t) in spans(text, tokens) {
        let start = offset;
        offset += t.encode_utf16().count();
        out.push(format!("{} {start} {offset}", kind.debug_name()));
    }
}

fn unhex(line: &str) -> String {
    let bytes = (0..line.len() / 2)
        .map(|i| u8::from_str_radix(&line[2 * i..2 * i + 2], 16).unwrap())
        .collect();
    String::from_utf8(bytes).unwrap()
}

/// The cases of one input file, as `(label, text)`.
fn cases(input: &Path) -> Vec<(String, String)> {
    let Ok(content) = std::fs::read_to_string(input) else {
        return Vec::new();
    };
    if input.extension().is_some_and(|e| e == "hexlines") {
        content
            .lines()
            .enumerate()
            .map(|(i, line)| (format!("# {i}"), unhex(line)))
            .collect()
    } else {
        vec![(String::new(), content)]
    }
}

#[test]
#[ignore = "needs KTRS_ORACLE_DIR with LexDump output"]
fn matches_jvm_lexers() {
    let dir = std::env::var("KTRS_ORACLE_DIR").expect("set KTRS_ORACLE_DIR");
    type Lex = fn(&str) -> Vec<Token>;
    let modes: [(&str, Lex); 2] = [
        ("kotlin", ktrs_lexer::tokenize),
        ("kdoc", ktrs_lexer::tokenize_kdoc),
    ];
    let (mut checked, mut failures) = (0, Vec::new());
    for input in walk(dir.as_ref(), &["hexlines", "kt", "kts"]) {
        let cases = cases(&input);
        for (mode, lex) in modes {
            let Ok(expected) = std::fs::read_to_string(format!("{}.{mode}.tok", input.display()))
            else {
                continue;
            };
            let mut groups: Vec<Vec<&str>> = vec![Vec::new()];
            for line in expected.lines() {
                if line.starts_with("# ") {
                    groups.push(Vec::new());
                } else {
                    groups.last_mut().unwrap().push(line);
                }
            }
            let groups = if groups.len() > 1 {
                &groups[1..]
            } else {
                &groups[..]
            };
            assert_eq!(groups.len(), cases.len(), "{} out of sync", input.display());
            for ((label, text), expected) in cases.iter().zip(groups) {
                checked += 1;
                let mut actual = Vec::new();
                dump(text, &lex(text), &mut actual);
                if let Some(i) = (0..expected.len().max(actual.len()))
                    .find(|&i| expected.get(i).copied() != actual.get(i).map(String::as_str))
                {
                    failures.push(format!(
                        "{} {label} [{mode}] {text:?} token {i}: expected {:?}, got {:?}",
                        input.display(),
                        expected.get(i),
                        actual.get(i)
                    ));
                }
            }
        }
    }
    eprintln!("{checked} cases checked, {} mismatches", failures.len());
    failures.truncate(30);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
