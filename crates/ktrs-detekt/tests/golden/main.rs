//! detekt's own rule tests as goldens: every `testdata/detekt/<RuleName>/<case>` (format in `case.rs`) runs the
//! ported rule with the case's config on the case's file and must return what the real rule returned from
//! `visitFile`: the same rows in the same order, or a panic where the rule threw. Cases come from
//! `tools/detekt-tests/extract-goldens.sh`. Cases of an unported rule are skipped and counted.
//! Ratchet: cases in `tests/golden-passing.txt` must pass; `UPDATE_PASSING=1 cargo test -p ktrs-detekt --release
//! --test golden` rewrites it. `GOLDEN_FILTER=<substring>` runs a subset (no ratchet check).

mod case;

use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::sync::Arc;
use std::{env, fs};

use case::{Case, TestConfig};
use ktrs_detekt::api::{Config, Finding, RuleProvider, config};
use ktrs_detekt::kt_file;
use ktrs_detekt::probe::escape;
use ktrs_detekt::rules::default_rule_set_providers;

const SHOWN_FAILURES: usize = 12;

/// Rules that report from a map hashed by identity upstream: their row order is not defined.
const UNORDERED: [&str; 1] = ["LongMethod"];

enum Outcome {
    Pass,
    Fail { expected: String, actual: String },
    /// The rule is not ported, or the case's config holds a value the recorder cannot express.
    Skip(String),
}

fn rule_provider(name: &str) -> Option<RuleProvider> {
    default_rule_set_providers()
        .iter()
        .flat_map(|provider| (provider.instance)().rules)
        .find(|(rule_name, _)| rule_name.value() == name)
        .map(|(_, provider)| provider)
}

fn row(finding: &Finding) -> String {
    let location = &finding.entity.location;
    format!(
        "{}\t{}\t{}\t{}\t{}",
        location.source,
        location.end_source,
        location.text,
        escape(&finding.entity.signature),
        escape(&finding.message)
    )
}

fn run_case(case: &Case) -> Outcome {
    let Some(provider) = rule_provider(&case.rule) else { return Outcome::Skip(case.rule.clone()) };
    let rule_config: Arc<dyn Config> = match &case.config {
        Ok(values) if values.is_empty() => config::empty(),
        Ok(values) => TestConfig::new(values.clone()),
        Err(reason) => return Outcome::Skip(reason.clone()),
    };
    let run = panic::catch_unwind(AssertUnwindSafe(|| {
        let file = kt_file::parse_kt_file(&case.input, &case.path);
        let _context = kt_file::enter(&file, Path::new(&case.path));
        let mut rule = provider(rule_config);
        rule.visit_root_file(&file).iter().map(row).collect::<Vec<_>>()
    }));
    let mut expected = case.findings.clone();
    let actual = match (run, &case.error) {
        (Err(_), Some(_)) => return Outcome::Pass,
        (Err(payload), None) => {
            let message = payload.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| payload.downcast_ref::<String>().cloned());
            return Outcome::Fail { expected: expected.join("\n"), actual: format!("panic: {}", message.unwrap_or_default()) };
        }
        (Ok(rows), Some(error)) => return Outcome::Fail { expected: format!("throws {error}"), actual: rows.join("\n") },
        (Ok(rows), None) => rows,
    };
    let mut actual = actual;
    if UNORDERED.contains(&case.rule.as_str()) {
        expected.sort();
        actual.sort();
    }
    if actual == expected { Outcome::Pass } else { Outcome::Fail { expected: expected.join("\n"), actual: actual.join("\n") } }
}

#[test]
fn detekt_goldens() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = crate_dir.join("../../testdata/detekt");
    let filter = env::var("GOLDEN_FILTER").ok();
    let cases: Vec<Case> = case::collect(&root)
        .iter()
        .map(|base| case::load(&root, base))
        .filter(|c| filter.as_ref().is_none_or(|f| c.name.contains(f.as_str())))
        .collect();
    assert!(!cases.is_empty(), "no cases under {}; run tools/detekt-tests/extract-goldens.sh", root.display());

    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let results: Vec<(&str, Outcome)> = cases.iter().map(|case| (case.name.as_str(), run_case(case))).collect();
    panic::set_hook(previous_hook);

    // per rule dir: pass, fail, skipped
    let mut per_rule: BTreeMap<&str, [usize; 3]> = BTreeMap::new();
    for (name, outcome) in &results {
        let counts = per_rule.entry(name.split('/').next().unwrap()).or_default();
        match outcome {
            Outcome::Pass => counts[0] += 1,
            Outcome::Fail { .. } => counts[1] += 1,
            Outcome::Skip(_) => counts[2] += 1,
        }
    }
    for (rule, [pass, fail, skip]) in &per_rule {
        println!("{rule}: passed {pass}, failed {fail}, skipped {skip}");
    }
    let total = |i: usize| per_rule.values().map(|c| c[i]).sum::<usize>();
    println!("golden: {} cases: passed {}, failed {}, skipped {}", results.len(), total(0), total(1), total(2));
    let failures = results.iter().filter_map(|(name, outcome)| match outcome {
        Outcome::Fail { expected, actual } => Some((name, expected, actual)),
        _ => None,
    });
    for (name, expected, actual) in failures.take(SHOWN_FAILURES) {
        println!("FAIL {name}\n--- expected\n{expected}\n--- actual\n{actual}");
    }

    let passed: Vec<&str> = results.iter().filter(|(_, o)| matches!(o, Outcome::Pass)).map(|(n, _)| *n).collect();
    if filter.is_some() {
        return;
    }
    let passing_path = crate_dir.join("tests/golden-passing.txt");
    if env::var_os("UPDATE_PASSING").is_some_and(|v| v == "1") {
        fs::write(&passing_path, passed.iter().map(|n| format!("{n}\n")).collect::<String>()).unwrap();
        println!("wrote {} entries to {}", passed.len(), passing_path.display());
        return;
    }
    let listed = fs::read_to_string(&passing_path).unwrap_or_default();
    let regressions: Vec<&str> = listed.lines().map(str::trim).filter(|n| !n.is_empty() && !passed.contains(n)).collect();
    assert!(regressions.is_empty(), "{} case(s) in tests/golden-passing.txt regressed:\n{}", regressions.len(), regressions.join("\n"));
}
