//! ktlint's own rule tests as goldens: every `testdata/ktlint/<rule-dir>/<case>` (format in `case.rs`) is linted
//! and formatted by ktrs-lint with the case's rules and editorconfig, and must reproduce the real engine's lint
//! rows, format-callback rows and formatted text exactly. Cases come from
//! `tools/ktlint-tests/extract-goldens.sh`. Cases needing an unported rule are skipped and counted.
//! Ratchet: cases in `tests/golden-passing.txt` must pass; `UPDATE_PASSING=1 cargo test -p ktrs-lint --release
//! --test golden` rewrites it. `GOLDEN_FILTER=<substring>` runs a subset (no ratchet check).
//! `GOLDEN_VERIFY_VISITED_TYPES=1` also runs every hook a rule's `visited_types` skips and fails the case if it acts.

mod case;
mod engine;
mod runner;

use std::env;
use std::path::Path;

use case::Case;
use ktrs_lint::rules::standard_rule_provider;

#[test]
fn ktlint_goldens() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = crate_dir.join("../../testdata/ktlint");
    let filter = env::var("GOLDEN_FILTER").ok();
    let cases: Vec<Case> = case::collect(&root)
        .iter()
        .map(|base| case::load(&root, base))
        .filter(|c| filter.as_ref().is_none_or(|f| c.name.contains(f.as_str())))
        .collect();
    assert!(!cases.is_empty(), "no cases under {}; run tools/ktlint-tests/extract-goldens.sh", root.display());
    ktrs_lint::engine::set_verify_visited_types(env::var_os("GOLDEN_VERIFY_VISITED_TYPES").is_some_and(|v| v == "1"));

    let outcomes = runner::run_all(&cases, |case| {
        runner::run_case(case, || engine::setup(&case.options, &case.input, &standard_rule_provider, &[], None))
    });
    let results: Vec<_> = cases.iter().map(|c| c.name.as_str()).zip(outcomes).collect();
    runner::print_summary(&results);
    runner::ratchet(&results, &crate_dir.join("tests/golden-passing.txt"), filter.is_some());
}
