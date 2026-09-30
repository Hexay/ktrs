//! Parses every `testdata/kotlin/psi/**/<name>.kt[s]` that has a sibling `<name>.txt` and compares
//! `psi_dump` with it. Ratchet: only fixtures listed in `tests/passing.txt` must pass;
//! `UPDATE_PASSING=1 cargo test -p ktrs-parser --test fixtures` rewrites that list.

use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::{env, fs};

use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::psi_dump;

const SHOWN_FAILURES: usize = 5;

enum Outcome {
    Pass,
    Fail { line: usize, expected: String, actual: String },
    Panic(String),
}

#[test]
fn psi_fixtures() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let psi_dir = crate_dir.join("../../testdata/kotlin/psi");
    let passing_path = crate_dir.join("tests/passing.txt");

    let mut fixtures = Vec::new();
    collect_fixtures(&psi_dir, &mut fixtures);
    fixtures.sort();
    assert!(!fixtures.is_empty(), "no fixtures under {}; run tools/sync-kotlin.sh", psi_dir.display());

    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let results: Vec<(String, Outcome)> =
        fixtures.iter().map(|source| (relative_name(&psi_dir, source), run_fixture(source))).collect();
    panic::set_hook(previous_hook);

    let passed: Vec<&str> =
        results.iter().filter(|(_, o)| matches!(o, Outcome::Pass)).map(|(name, _)| name.as_str()).collect();
    println!("passed {}/{}", passed.len(), results.len());
    for (name, outcome) in results.iter().filter(|(_, o)| !matches!(o, Outcome::Pass)).take(SHOWN_FAILURES) {
        match outcome {
            Outcome::Fail { line, expected, actual } => {
                println!("FAIL {name}:{line}\n  expected: {expected}\n  actual:   {actual}")
            }
            Outcome::Panic(message) => println!("PANIC {name}: {message}"),
            Outcome::Pass => unreachable!(),
        }
    }

    if env::var_os("UPDATE_PASSING").is_some_and(|v| v == "1") {
        let mut list = passed.join("\n");
        list.push('\n');
        fs::write(&passing_path, list).unwrap();
        println!("wrote {} entries to {}", passed.len(), passing_path.display());
        return;
    }

    let listed = fs::read_to_string(&passing_path).unwrap_or_default();
    let regressions: Vec<&str> = listed
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty() && !passed.contains(name))
        .collect();
    assert!(regressions.is_empty(), "{} fixture(s) in tests/passing.txt regressed:\n{}", regressions.len(), regressions.join("\n"));
}

fn collect_fixtures(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_fixtures(&path, out);
        } else if matches!(path.extension().and_then(|e| e.to_str()), Some("kt" | "kts"))
            && path.with_extension("txt").is_file()
        {
            out.push(path);
        }
    }
}

fn relative_name(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/")
}

fn run_fixture(source: &Path) -> Outcome {
    let file_name = source.file_name().unwrap().to_str().unwrap();
    // Upstream's test framework feeds the text with CRLF -> LF and trailing newlines stripped.
    let text = fs::read_to_string(source).unwrap().replace("\r\n", "\n");
    let text = text.trim_end_matches('\n');
    let expected = fs::read_to_string(source.with_extension("txt")).unwrap().replace("\r\n", "\n");

    let actual = match panic::catch_unwind(AssertUnwindSafe(|| {
        psi_dump(&parse_file(text, FileKind::from_file_name(file_name)), file_name)
    })) {
        Ok(dump) => dump,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic>".to_owned());
            return Outcome::Panic(message);
        }
    };
    first_difference(expected.trim_end(), actual.trim_end())
}

fn first_difference(expected: &str, actual: &str) -> Outcome {
    let (mut e, mut a) = (expected.lines(), actual.lines());
    let mut line = 1;
    loop {
        match (e.next(), a.next()) {
            (None, None) => return Outcome::Pass,
            (x, y) if x == y => line += 1,
            (x, y) => {
                return Outcome::Fail {
                    line,
                    expected: x.unwrap_or("<eof>").to_owned(),
                    actual: y.unwrap_or("<eof>").to_owned(),
                };
            }
        }
    }
}
