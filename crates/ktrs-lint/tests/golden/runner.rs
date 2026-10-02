//! Running golden cases: one case against the real engine's rows and text, all cases on every core, the summary
//! and the `golden-passing.txt` ratchet. Shared with `crates/ktrs-compose/tests/golden` (`#[path]`).

use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, fs, thread};

use ktrs_lint::{AutocorrectDecision, Code, KtLintRuleEngine, LintError};

use crate::case::Case;
use crate::engine::Skip;

const SHOWN_FAILURES: usize = 8;

pub enum Outcome {
    Pass,
    Fail { what: &'static str, expected: String, actual: String },
    Panic(String),
    Skip(Skip),
}

/// `f` over all `items` on every core, results in item order.
pub fn run_all<T: Sync>(items: &[T], f: impl Fn(&T) -> Outcome + Sync) -> Vec<Outcome> {
    let next = AtomicUsize::new(0);
    let threads = thread::available_parallelism().map_or(4, |n| n.get());
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let mut results: Vec<(usize, Outcome)> = thread::scope(|s| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                thread::Builder::new()
                    .stack_size(64 << 20)
                    .spawn_scoped(s, || {
                        let mut out = Vec::new();
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some(item) = items.get(i) else { return out };
                            out.push((i, f(item)));
                        }
                    })
                    .unwrap()
            })
            .collect();
        workers.into_iter().flat_map(|w| w.join().unwrap()).collect()
    });
    panic::set_hook(previous_hook);
    results.sort_by_key(|(i, _)| *i);
    results.into_iter().map(|(_, o)| o).collect()
}

fn row(e: &LintError) -> String {
    let auto = if e.can_be_auto_corrected { "auto" } else { "manual" };
    format!("{}:{}\t{}\t{auto}\t{}", e.line, e.col, e.rule_id.value(), e.detail.replace('\\', "\\\\").replace('\n', "\\n"))
}

/// Lints and formats `case` with the engine `setup` builds, against the case's expectations.
pub fn run_case(case: &Case, setup: impl FnOnce() -> Result<(KtLintRuleEngine, Code), Skip>) -> Outcome {
    let (engine, code) = match setup() {
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

/// Without a filter: rewrites the ratchet (`UPDATE_PASSING=1`) or asserts every listed case still passes.
pub fn ratchet(results: &[(&str, Outcome)], passing_path: &Path, filtered: bool) {
    let passed: Vec<&str> = results.iter().filter(|(_, o)| matches!(o, Outcome::Pass)).map(|(n, _)| *n).collect();
    if filtered {
        return;
    }
    if env::var_os("UPDATE_PASSING").is_some_and(|v| v == "1") {
        fs::write(passing_path, passed.iter().map(|n| format!("{n}\n")).collect::<String>()).unwrap();
        println!("wrote {} entries to {}", passed.len(), passing_path.display());
        return;
    }
    let listed = fs::read_to_string(passing_path).unwrap_or_default();
    let regressions: Vec<&str> = listed.lines().map(str::trim).filter(|n| !n.is_empty() && !passed.contains(n)).collect();
    let name = passing_path.file_name().unwrap().to_string_lossy();
    assert!(regressions.is_empty(), "{} case(s) in tests/{name} regressed:\n{}", regressions.len(), regressions.join("\n"));
}

pub fn print_summary(results: &[(&str, Outcome)]) {
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
