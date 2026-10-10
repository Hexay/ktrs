//! Builders that stop before their last token (research/24, finding 4): the tree leaves the tokens out, as
//! IntelliJ's, and the parse reports them for the callers to raise IntelliJ's `AssertionError`.

use std::path::{Path, PathBuf};
use std::{env, fs};

use ktrs_parser::{ChameleonCache, FileKind, parse_file, parse_file_cached};

const FUZZ_MINIMAL: &str = "{fun<)]<T:@( {})";

#[test]
fn lambda_reparse_that_stops_early_reports_the_jvm_text() {
    for kind in [FileKind::Source, FileKind::Script] {
        let parse = parse_file(FUZZ_MINIMAL, kind);
        assert_ne!(parse.tree.text(), FUZZ_MINIMAL);
        assert_eq!(parse.missed_tokens.len(), 1);
        let missed = parse.first_missed_tokens().unwrap();
        assert_ne!(missed.element, 0, "a chameleon's, not the file's");
        assert_eq!(missed.text, FUZZ_MINIMAL);
        // The JVM's (ktfmt 0.65 and ktlint 2.0.0-ALPHA-4 jars) stderr and exception, stack trace aside.
        assert_eq!(missed.log(), "ERROR: Tokens [RPAR] were not inserted into the tree. Language: kotlin\nDetails:\nmissedTokensFragment.txt\n{fun<)]<T:@( {})");
        assert_eq!(missed.assertion_error(), "java.lang.AssertionError: Tokens [RPAR] were not inserted into the tree. Language: kotlin");
    }
}

#[test]
fn cached_parses_report_missed_tokens_too() {
    let mut cache = ChameleonCache::new();
    for _ in 0..2 {
        assert_eq!(parse_file_cached(FUZZ_MINIMAL, FileKind::Script, &mut cache).missed_tokens, parse_file(FUZZ_MINIMAL, FileKind::Script).missed_tokens);
    }
}

/// Upstream's parser tests fail on any `LOG.error`, so no fixture may miss tokens.
#[test]
fn no_fixture_misses_tokens() {
    let mut files = Vec::new();
    collect(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/kotlin/psi"), &mut files);
    assert!(!files.is_empty());
    for file in &files {
        let text = fs::read_to_string(file).unwrap().replace("\r\n", "\n");
        let parse = parse_file(text.trim_end_matches('\n'), FileKind::from_file_name(&file.to_string_lossy()));
        assert_eq!(parse.missed_tokens, [], "{}", file.display());
    }
}

/// The same over real code: `KTRS_CORPUS=<dir>` (e.g. `corpus/`); skipped without it.
#[test]
fn no_corpus_file_misses_tokens() {
    let Some(dir) = env::var_os("KTRS_CORPUS") else { return };
    let mut files = Vec::new();
    collect(Path::new(&dir), &mut files);
    for file in &files {
        let Ok(text) = fs::read_to_string(file) else { continue };
        let parse = parse_file(&text.replace("\r\n", "\n"), FileKind::from_file_name(&file.to_string_lossy()));
        assert_eq!(parse.missed_tokens, [], "{}", file.display());
    }
    eprintln!("{} corpus files, none misses tokens", files.len());
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, out);
        } else if matches!(path.extension().and_then(|e| e.to_str()), Some("kt" | "kts")) {
            out.push(path);
        }
    }
}
