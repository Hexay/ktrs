//! Robustness + throughput over a real-world corpus:
//! `KTRS_CORPUS=<dir> cargo test --release -p ktrs_lexer --test corpus -- --ignored --nocapture`

mod common;

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};

use common::{convert_line_separators, spans, walk};
use ktrs_syntax::SyntaxKind;

#[test]
#[ignore = "needs KTRS_CORPUS=<dir of .kt/.kts files>"]
fn corpus_is_lossless() {
    let dir = std::env::var("KTRS_CORPUS").expect("set KTRS_CORPUS to a directory");
    let files = walk(dir.as_ref(), &["kt", "kts"]);
    let (mut bytes, mut kdoc_bytes, mut non_utf8) = (0usize, 0usize, 0usize);
    let (mut elapsed, mut kdoc_elapsed) = (Duration::ZERO, Duration::ZERO);
    let mut failures = Vec::new();
    for path in &files {
        let Ok(text) = String::from_utf8(std::fs::read(path).unwrap()) else {
            non_utf8 += 1;
            continue;
        };
        let text = convert_line_separators(&text);
        let result = catch_unwind(AssertUnwindSafe(|| {
            let start = Instant::now();
            let tokens = ktrs_lexer::tokenize(&text);
            let lexed = start.elapsed();
            let mut kdoc = (0, Duration::ZERO);
            for (_, doc) in spans(&text, &tokens)
                .into_iter()
                .filter(|(k, _)| *k == SyntaxKind::DOC_COMMENT)
            {
                let start = Instant::now();
                let kdoc_tokens = ktrs_lexer::tokenize_kdoc(doc);
                kdoc.1 += start.elapsed();
                kdoc.0 += doc.len();
                spans(doc, &kdoc_tokens);
            }
            (lexed, kdoc)
        }));
        match result {
            Ok((lexed, (doc_len, doc_time))) => {
                bytes += text.len();
                elapsed += lexed;
                kdoc_bytes += doc_len;
                kdoc_elapsed += doc_time;
            }
            Err(_) => failures.push(path.display().to_string()),
        }
    }
    let mb_per_s = |b: usize, d: Duration| b as f64 / 1e6 / d.as_secs_f64().max(1e-9);
    eprintln!(
        "{} files ({non_utf8} non-UTF-8 skipped), {:.1} MB: Kotlin {:.0} MB/s; KDoc {:.1} MB at {:.0} MB/s",
        files.len(),
        bytes as f64 / 1e6,
        mb_per_s(bytes, elapsed),
        kdoc_bytes as f64 / 1e6,
        mb_per_s(kdoc_bytes, kdoc_elapsed),
    );
    assert!(
        failures.is_empty(),
        "{} file(s) panicked or were not lossless:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
