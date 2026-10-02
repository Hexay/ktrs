//! compose-rules' own rule tests as goldens: every `testdata/compose-rules/<rule-dir>/<case>` (format of
//! `crates/ktrs-lint/tests/golden/case.rs`, recorded on ktlint 1.8.0 with the compose-rules JAR) runs twice: in 1.8
//! mode against its rows and text, and in 2.0 mode against the `.2_0.*` expectations (`variant.rs`). Rule ids
//! resolve over the standard and compose providers; cases of unported rules are skipped and counted.
//! Ratchet: `<case>@1.8` / `<case>@2.0` entries of `tests/golden-passing.txt` must pass; `UPDATE_PASSING=1 cargo test
//! -p ktrs-compose --release --test golden` rewrites it. `GOLDEN_FILTER=<substring>` runs a subset (no ratchet).

#[path = "../../../ktrs-lint/tests/golden/case.rs"]
mod case;
#[path = "../../../ktrs-lint/tests/golden/engine.rs"]
mod engine;
#[path = "../../../ktrs-lint/tests/golden/runner.rs"]
mod runner;
mod variant;

use std::env;
use std::path::Path;

use case::Case;
use ktrs_lint::RuleV2Provider;
use ktrs_lint::editorconfig::KtlintVersion;
use ktrs_lint::rules::standard_rule_provider;

struct Run {
    name: String,
    case: Case,
    version: KtlintVersion,
}

#[test]
fn compose_rules_goldens() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = crate_dir.join("../../testdata/compose-rules");
    if !root.is_dir() {
        println!("no goldens under {}: nothing to check until the compose-rules goldens land", root.display());
        return;
    }
    let filter = env::var("GOLDEN_FILTER").ok();
    let runs: Vec<Run> = case::collect(&root)
        .iter()
        .map(|base| case::load(&root, base))
        .flat_map(|case_18| {
            let case_20 = variant::ktlint_2_0(&root, &case_18);
            [
                Run { name: format!("{}@1.8", case_18.name), case: case_18, version: KtlintVersion::V1_8 },
                Run { name: format!("{}@2.0", case_20.name), case: case_20, version: KtlintVersion::V2_0 },
            ]
        })
        .filter(|r| filter.as_ref().is_none_or(|f| r.name.contains(f.as_str())))
        .collect();

    let compose = ktrs_compose::compose_rule_providers();
    let resolve = |id: &str| -> Option<RuleV2Provider> {
        standard_rule_provider(id).or_else(|| compose.iter().find(|p| p.rule_id().value() == id).cloned())
    };
    let outcomes = runner::run_all(&runs, |run| {
        runner::run_case(&run.case, || engine::setup(&run.case.options, &run.case.input, &resolve, &compose, Some(run.version)))
    });
    let results: Vec<_> = runs.iter().map(|r| r.name.as_str()).zip(outcomes).collect();
    runner::print_summary(&results);
    runner::ratchet(&results, &crate_dir.join("tests/golden-passing.txt"), filter.is_some());
}
