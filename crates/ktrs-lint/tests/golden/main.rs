//! ktlint's own rule tests as goldens: every `testdata/ktlint/<rule-dir>/<case>` (format in `case.rs`) is linted
//! and formatted by ktrs-lint with the case's rules and editorconfig, and must reproduce the real engine's lint
//! rows, format-callback rows and formatted text exactly. Cases come from
//! `tools/ktlint-tests/extract-goldens.sh`. Cases needing an unported rule are skipped and counted.
//! Ratchet: cases in `tests/golden-passing.txt` must pass; `UPDATE_PASSING=1 cargo test -p ktrs-lint --release
//! --test golden` rewrites it. `GOLDEN_FILTER=<substring>` runs a subset (no ratchet check).

mod case;
mod engine;

use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::{env, fs};

use case::Case;
use engine::Skip;
use ktrs_lint::{AutocorrectDecision, LintError};

const SHOWN_FAILURES: usize = 8;

enum Outcome {
    Pass,
    Fail { what: &'static str, expected: String, actual: String },
    Panic(String),
    Skip(Skip),
}

#[test]
fn ktlint_goldens() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = crate_dir.join("../../testdata/ktlint");
    let passing_path = crate_dir.join("tests/golden-passing.txt");
    let filter = env::var("GOLDEN_FILTER").ok();
    let cases: Vec<Case> = case::collect(&root)
        .iter()
        .map(|base| case::load(&root, base))
        .filter(|c| filter.as_ref().is_none_or(|f| c.name.contains(f.as_str())))
        .collect();
    assert!(!cases.is_empty(), "no cases under {}; run tools/ktlint-tests/extract-goldens.sh", root.display());

    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let results: Vec<(&str, Outcome)> = cases.iter().map(|c| (c.name.as_str(), run_case(c))).collect();
    panic::set_hook(previous_hook);

    print_summary(&results);
    let passed: Vec<&str> = results.iter().filter(|(_, o)| matches!(o, Outcome::Pass)).map(|(n, _)| *n).collect();
    if filter.is_some() {
        return;
    }
    if env::var_os("UPDATE_PASSING").is_some_and(|v| v == "1") {
        fs::write(&passing_path, passed.iter().map(|n| format!("{n}\n")).collect::<String>()).unwrap();
        println!("wrote {} entries to {}", passed.len(), passing_path.display());
        return;
    }
    let listed = fs::read_to_string(&passing_path).unwrap_or_default();
    let regressions: Vec<&str> = listed.lines().map(str::trim).filter(|n| !n.is_empty() && !passed.contains(n)).collect();
    assert!(regressions.is_empty(), "{} case(s) in tests/golden-passing.txt regressed:\n{}", regressions.len(), regressions.join("\n"));
}

fn row(e: &LintError) -> String {
    let auto = if e.can_be_auto_corrected { "auto" } else { "manual" };
    format!("{}:{}\t{}\t{auto}\t{}", e.line, e.col, e.rule_id.value(), e.detail.replace('\\', "\\\\").replace('\n', "\\n"))
}

fn run_case(case: &Case) -> Outcome {
    let (engine, code) = match engine::setup(&case.options, &case.input) {
        Ok(setup) => setup,
        Err(skip) => return Outcome::Skip(skip),
    };
    let run = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut lint = Vec::new();
        let lint_result = engine.lint(&code, &mut |e| lint.push(row(e)));
        let mut format = Vec::new();
        let formatted = engine.format(&code, &mut |e| {
            format.push(row(e));
            AutocorrectDecision::AllowAutocorrect
        });
        (lint_result.map(|()| lint), formatted.map(|text| (format, text)))
    }));
    let (lint, format) = match run {
        Ok(results) => results,
        Err(payload) => {
            let message = payload.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| payload.downcast_ref::<String>().cloned());
            return Outcome::Panic(message.unwrap_or_else(|| "<non-string panic>".to_owned()));
        }
    };
    let error = |stage: &str| case.errors.iter().find(|(s, _)| s == stage).map(|(_, e)| e.clone());
    match (error("lint"), lint) {
        (None, Ok(rows)) if rows != case.lint => return fail("lint", case.lint.join("\n"), rows.join("\n")),
        (None, Err(e)) => return fail("lint", case.lint.join("\n"), format!("{e:?}")),
        (Some(expected), Ok(rows)) => return fail("lint", expected, rows.join("\n")),
        _ => {}
    }
    match (error("format"), format) {
        (None, Ok((rows, _))) if rows != case.format => fail("format callback", case.format.join("\n"), rows.join("\n")),
        (None, Ok((_, text))) if text != case.expected => fail("formatted text", case.expected.clone(), text),
        (None, Err(e)) => fail("format", case.expected.clone(), format!("{e:?}")),
        (Some(expected), Ok((_, text))) => fail("format", expected, text),
        _ => Outcome::Pass,
    }
}

fn fail(what: &'static str, expected: String, actual: String) -> Outcome {
    Outcome::Fail { what, expected, actual }
}

fn print_summary(results: &[(&str, Outcome)]) {
    // per rule dir: pass, fail (panics included), skipped
    let mut per_rule: BTreeMap<&str, [usize; 3]> = BTreeMap::new();
    let mut missing: BTreeMap<&str, usize> = BTreeMap::new();
    let mut unsupported_ec: BTreeMap<&str, usize> = BTreeMap::new();
    for (name, outcome) in results {
        let counts = per_rule.entry(name.split('/').next().unwrap()).or_default();
        match outcome {
            Outcome::Pass => counts[0] += 1,
            Outcome::Fail { .. } | Outcome::Panic(_) => counts[1] += 1,
            Outcome::Skip(skip) => {
                counts[2] += 1;
                match skip {
                    Skip::Rule(id) => *missing.entry(id.as_str()).or_default() += 1,
                    Skip::EditorConfig(p) => *unsupported_ec.entry(p.split('=').next().unwrap()).or_default() += 1,
                }
            }
        }
    }
    for (rule, [pass, fail, skip]) in per_rule.iter().filter(|(_, c)| c[0] + c[1] > 0) {
        println!("{rule}: passed {pass}, failed {fail}, skipped {skip}");
    }
    let total = |i: usize| per_rule.values().map(|c| c[i]).sum::<usize>();
    let rule_skips = missing.values().sum::<usize>();
    println!(
        "golden: {} cases: passed {}, failed {}, skipped {} (rule not ported: {rule_skips} cases, {} rules; editorconfig not supported: {})",
        results.len(),
        total(0),
        total(1),
        total(2),
        missing.len(),
        total(2) - rule_skips,
    );
    let mut top: Vec<_> = missing.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let top: Vec<String> = top.iter().take(10).map(|(id, n)| format!("{id} {n}")).collect();
    println!("most-needed unported rules (cases): {}", top.join(", "));
    if !unsupported_ec.is_empty() {
        println!("unsupported editorconfig overrides (cases): {unsupported_ec:?}");
    }
    let failures = results.iter().filter(|(_, o)| matches!(o, Outcome::Fail { .. } | Outcome::Panic(_)));
    for (name, outcome) in failures.take(SHOWN_FAILURES) {
        match outcome {
            Outcome::Fail { what, expected, actual } => println!("FAIL {name}: {what}\n--- expected\n{expected}\n--- actual\n{actual}"),
            Outcome::Panic(message) => println!("PANIC {name}: {message}"),
            _ => unreachable!(),
        }
    }
}
