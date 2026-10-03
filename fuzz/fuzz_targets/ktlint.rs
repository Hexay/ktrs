//! ktlint port: lint and format (autocorrect everything) with every ported standard rule. Rule panics with a
//! Java-exception message are ktlint's own `KtLintRuleException`s; any other panic, even one the engine catches at
//! the rule boundary, is a finding.
#![no_main]

use ktrs_lint::editorconfig::{CODE_STYLE_PROPERTY, EXPERIMENTAL_RULES_EXECUTION_PROPERTY, PropertyRef};
use ktrs_lint::rules::standard_rule_providers;
use ktrs_lint::{AutocorrectDecision, Code, EditorConfigDefaults, EditorConfigOverride, KtLintRuleEngine};
use libfuzzer_sys::fuzz_target;

ktrs_fuzz::splice_mutators!();

/// (code style, experimental rules): one is picked per input by hash, so each input always takes the same path.
const CONFIGS: [(&str, bool); 4] =
    [("ktlint_official", false), ("ktlint_official", true), ("intellij_idea", false), ("android_studio", false)];

thread_local! {
    static ENGINES: Vec<KtLintRuleEngine> = CONFIGS.iter().map(|&(style, experimental)| engine(style, experimental)).collect();
}

fn engine(style: &str, experimental: bool) -> KtLintRuleEngine {
    let mut properties = vec![(PropertyRef::from(&*CODE_STYLE_PROPERTY), Some(style.to_owned()))];
    if experimental {
        properties.push((PropertyRef::from(&*EXPERIMENTAL_RULES_EXECUTION_PROPERTY), Some("enabled".to_owned())));
    }
    KtLintRuleEngine::with_editor_config(
        standard_rule_providers(),
        EditorConfigDefaults::empty(),
        EditorConfigOverride::from(properties),
    )
}

fn fnv(data: &[u8]) -> usize {
    data.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)) as usize
}

fuzz_target!(|data: &[u8]| {
    let Some(text) = ktrs_fuzz::fuzz_input(data) else { return };
    let pick = fnv(data);
    let code = Code::from_snippet(text, pick / CONFIGS.len() % 2 == 1);
    ENGINES.with(|engines| {
        let engine = &engines[pick % CONFIGS.len()];
        let _ = ktrs_fuzz::tolerate_java_panics(|| engine.lint(&code, &mut |_| {}));
        let _ = ktrs_fuzz::tolerate_java_panics(|| {
            engine.format(&code, &mut |_| AutocorrectDecision::AllowAutocorrect)
        });
    });
});
