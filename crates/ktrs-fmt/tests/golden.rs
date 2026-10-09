//! Runs every `testdata/ktfmt/<group>/<case>.input.kt` through `ktrs_fmt::format` with the file type
//! and options in `<case>.options` and requires byte-identical `<case>.expected.kt`, or an `Err` when
//! `<case>.error` exists (ktfmt rejected the input). Cases come from ktfmt's own file-based tests,
//! re-derived with the real jar: `tools/ktfmt-oracle/extract-goldens.sh`. Ratchet: only cases listed in
//! `tests/golden-passing.txt` must pass; `UPDATE_PASSING=1 cargo test -p ktrs-fmt --test golden`
//! rewrites that list.

use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::{env, fs};

use ktrs_fmt::{FileType, FormattingOptions, META_FORMAT, TrailingCommaManagementStrategy, format};

const SHOWN_FAILURES: usize = 5;

enum Outcome {
    Pass,
    Fail { line: usize, expected: String, actual: String },
    Panic(String),
}

#[test]
fn ktfmt_goldens() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cases_dir = crate_dir.join("../../testdata/ktfmt");
    let passing_path = crate_dir.join("tests/golden-passing.txt");

    let mut cases = Vec::new();
    collect_cases(&cases_dir, &mut cases);
    cases.sort();
    assert!(!cases.is_empty(), "no cases under {}; run tools/ktfmt-oracle/extract-goldens.sh", cases_dir.display());

    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let results: Vec<(String, Outcome)> = cases.iter().map(|base| (relative_name(&cases_dir, base), run_case(base))).collect();
    panic::set_hook(previous_hook);

    let mut per_suite: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for (name, outcome) in &results {
        let entry = per_suite.entry(name.split('/').next().unwrap()).or_default();
        entry.0 += matches!(outcome, Outcome::Pass) as usize;
        entry.1 += 1;
    }
    for (suite, (passed, total)) in &per_suite {
        println!("{suite}: passed {passed}/{total}");
    }
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
    let regressions: Vec<&str> =
        listed.lines().map(str::trim).filter(|name| !name.is_empty() && !passed.contains(name)).collect();
    assert!(
        regressions.is_empty(),
        "{} case(s) in tests/golden-passing.txt regressed:\n{}",
        regressions.len(),
        regressions.join("\n")
    );
}

/// Collects case base paths (`<dir>/<case>`, without the `.input.kt` suffix).
fn collect_cases(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_cases(&path, out);
        } else if let Some(base) = path.to_str().and_then(|p| p.strip_suffix(".input.kt")) {
            out.push(PathBuf::from(base));
        }
    }
}

fn relative_name(root: &Path, base: &Path) -> String {
    base.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/")
}

fn sibling(base: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!("{}{suffix}", base.display()))
}

fn run_case(base: &Path) -> Outcome {
    // Inputs are fed verbatim: ktfmt itself handles CRLF and the shebang line.
    let input = fs::read_to_string(sibling(base, ".input.kt")).unwrap();
    let options_text = fs::read_to_string(sibling(base, ".options")).unwrap();
    let expected = fs::read_to_string(sibling(base, ".expected.kt")).ok();

    let (file_type, options) = parse_options(&options_text);
    let result = match panic::catch_unwind(AssertUnwindSafe(|| format(&input, file_type, &options))) {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic>".to_owned());
            return Outcome::Panic(message);
        }
    };
    match (expected, result) {
        (Some(expected), Ok(actual)) => first_difference(&expected, &actual),
        (None, Err(_)) => Outcome::Pass,
        (Some(_), Err(e)) => {
            Outcome::Fail { line: 0, expected: "<formatted output>".to_owned(), actual: format!("error {e}") }
        }
        (None, Ok(_)) => Outcome::Fail {
            line: 0,
            expected: fs::read_to_string(sibling(base, ".error")).unwrap_or_default().trim_end().to_owned(),
            actual: "<formatted output>".to_owned(),
        },
    }
}

/// Maps the `key=value` lines written by the extractor (`fileType` and ktfmt's `FormattingOptions` property names).
fn parse_options(text: &str) -> (FileType, FormattingOptions) {
    let mut options = META_FORMAT;
    let mut file_type = FileType::Regular;
    for (key, value) in text.lines().filter_map(|l| l.split_once('=')) {
        let (key, value) = (key.trim(), value.trim());
        let number = || value.parse::<i32>().unwrap_or_else(|_| panic!("bad {key}={value}"));
        let flag = || value == "true";
        match key {
            "fileType" => {
                file_type = match value {
                    "REGULAR" => FileType::Regular,
                    "SCRIPT" => FileType::Script,
                    _ => panic!("bad {key}={value}"),
                }
            }
            "maxWidth" => options.max_width = number(),
            "blockIndent" => options.block_indent = number(),
            "continuationIndent" => options.continuation_indent = number(),
            "trailingCommaManagementStrategy" => {
                options.trailing_comma_management_strategy =
                    TrailingCommaManagementStrategy::value_of(value).unwrap_or_else(|| panic!("bad {key}={value}"))
            }
            "removeUnusedImports" => options.remove_unused_imports = flag(),
            "preserveLambdaBreaks" => options.preserve_lambda_breaks = flag(),
            "debuggingPrintOpsAfterFormatting" => options.debugging_print_ops_after_formatting = flag(),
            _ => panic!("unknown option {key}"),
        }
    }
    (file_type, options)
}

fn first_difference(expected: &str, actual: &str) -> Outcome {
    if expected == actual {
        return Outcome::Pass;
    }
    let (mut e, mut a) = (expected.split('\n'), actual.split('\n'));
    let mut line = 1;
    loop {
        match (e.next(), a.next()) {
            (x, y) if x == y && x.is_some() => line += 1,
            (x, y) => {
                return Outcome::Fail {
                    line,
                    expected: x.map_or("<eof>".to_owned(), |s| format!("{s:?}")),
                    actual: y.map_or("<eof>".to_owned(), |s| format!("{s:?}")),
                };
            }
        }
    }
}
