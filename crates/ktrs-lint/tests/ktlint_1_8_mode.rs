//! `ktrs_ktlint_version`: one fixture per 1.8/2.0 switch in `tests/data/ktlint_1_8_mode/` (R1-R6 and the smaller
//! fixes of research/26-ktlint-18-mode.md), linted in each mode against the rows of that release's jar
//! (`ktlint-<version> --relative` in the fixture directory, sorted; `expected-<version>.txt`).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ktrs_lint::editorconfig::{KTLINT_VERSION_PROPERTY, KtlintVersion};
use ktrs_lint::rules::standard_rule_providers;
use ktrs_lint::{Code, EditorConfigDefaults, EditorConfigOverride, KtLintRuleEngine};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/ktlint_1_8_mode")
}

/// The plain reporter's `--relative` rows of every fixture, sorted like `LC_ALL=C sort`.
fn rows(engine: &KtLintRuleEngine) -> Vec<String> {
    let mut rows = rows_in_order(engine);
    rows.sort();
    rows
}

/// The rows file by file, each file's in the engine's order.
fn rows_in_order(engine: &KtLintRuleEngine) -> Vec<String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "kt"))
        .collect();
    files.sort();
    let mut rows = Vec::new();
    for file in files {
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        let code = Code::from_file(&file).unwrap();
        engine
            .lint(&code, &mut |e| rows.push(format!("{name}:{}:{}: {} ({})", e.line, e.col, e.detail, e.rule_id)))
            .unwrap();
    }
    rows
}

fn expected(release: &str) -> Vec<String> {
    let text = std::fs::read_to_string(fixtures().join(format!("expected-{release}.txt"))).unwrap();
    text.lines().map(str::to_owned).collect()
}

fn engine_in(ktlint_version: KtlintVersion) -> KtLintRuleEngine {
    let editor_config_override = EditorConfigOverride::empty().with(&KTLINT_VERSION_PROPERTY, ktlint_version);
    KtLintRuleEngine::with_editor_config(standard_rule_providers(), EditorConfigDefaults::empty(), editor_config_override)
}

#[test]
fn version_1_8_lints_like_the_1_8_jar() {
    assert_eq!(rows(&engine_in(KtlintVersion::V1_8)), expected("1.8"));
}

/// Errors at one position in 1.8's rule-major order (`expected-1.8-order.txt`: the jar's rows, grouped by file
/// with a stable sort, so each file keeps the jar's order).
#[test]
fn version_1_8_orders_errors_at_one_position_like_the_jar() {
    assert_eq!(rows_in_order(&engine_in(KtlintVersion::V1_8)), expected("1.8-order"));
}

#[test]
fn version_2_0_lints_like_the_2_0_jar() {
    assert_eq!(rows(&engine_in(KtlintVersion::V2_0)), expected("2.0"));
}

#[test]
fn without_the_property_the_engine_is_2_0() {
    assert_eq!(rows(&KtLintRuleEngine::new(standard_rule_providers())), expected("2.0"));
}

#[test]
fn an_invalid_version_counts_as_2_0() {
    let engine = KtLintRuleEngine::with_editor_config(
        standard_rule_providers(),
        EditorConfigDefaults::empty(),
        EditorConfigOverride::from(vec![((&*KTLINT_VERSION_PROPERTY).into(), Some("1.7".to_owned()))]),
    );
    assert_eq!(rows(&engine), expected("2.0"));
}

/// Also: the version read from the file's own `.editorconfig`; 2.0 dropped the warning.
#[test]
fn version_1_8_warns_about_obsolete_disabled_rules_properties() {
    let warnings_in = |version: &str| {
        let dir = std::env::temp_dir().join(format!("ktrs-obsolete-{version}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let editor_config = format!("root = true\n[*.kt]\ndisabled_rules = no-semi\nktrs_ktlint_version = {version}\n");
        std::fs::write(dir.join(".editorconfig"), editor_config).unwrap();
        std::fs::write(dir.join("A.kt"), "val a = 1\n").unwrap();
        let warnings = Arc::new(Mutex::new(Vec::new()));
        let sink = warnings.clone();
        let engine = KtLintRuleEngine::new(standard_rule_providers())
            .with_engine_warnings(Arc::new(move |logger, message| sink.lock().unwrap().push(format!("{logger} -- {message}"))));
        engine.lint(&Code::from_file(&dir.join("A.kt")).unwrap(), &mut |_| {}).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        let warnings = warnings.lock().unwrap().clone();
        warnings
    };
    assert_eq!(
        warnings_in("1.8"),
        ["com.pinterest.ktlint.rule.engine.internal.RuleExecutionContext -- Editorconfig property 'disabled_rules' is obsolete and is \
          not used by KtLint starting from version 0.49. Remove the property from all '.editorconfig' files."]
    );
    assert!(warnings_in("2.0").is_empty());
}
