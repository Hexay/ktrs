//! Port of mrmans0n/compose-rules (pin: `tools/sync-compose-rules.sh`), its ktlint flavour
//! (`io.nlopez.compose.rules:ktlint`): the rules of `ComposeRuleSetProvider` (rule set `compose`), run natively
//! when a ktlint run loads that release's JAR (`-R`); every other rule set JAR goes to the real ktlint jar.
//! Design, parity gates and how to bump the pin: research/27-custom-rulesets-impl.md.
//!
//! Layout mirrors upstream: `core` = `rules/common/.../core`, `rules` = `rules/common/.../rules`, `ktlint` =
//! `rules/ktlint` (the `KtlintRule` adapter, `.editorconfig` properties, the `*Check` classes, the provider).

pub mod ktlint;

pub use ktlint::compose_rule_set_provider::compose_rule_providers;

/// The ported release.
pub const COMPOSE_RULES_VERSION: &str = "0.6.7";

/// (release, content fingerprint) of the ktlint JARs that run natively; the fingerprint is computed by
/// `ktrs-cli`'s `compose_jar.rs` (published `ktlint-compose-<release>-all.jar`).
pub const NATIVE_JARS: &[(&str, &str)] = &[("0.6.7", "699dddb23bee236589a86454ead298daf2103206f4ca7432589864135b496ed3")];
