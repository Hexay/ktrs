//! Port of mrmans0n/compose-rules (pin: `tools/sync-compose-rules.sh`), its ktlint flavour
//! (`io.nlopez.compose.rules:ktlint`): the rules of `ComposeRuleSetProvider` (rule set `compose`), run natively
//! when a ktlint run loads that release's JAR (`-R`); every other rule set JAR goes to the real ktlint jar.
//!
//! Layout mirrors upstream, one Rust file per Kotlin file and one fn per Kotlin fn in the same order: `core` =
//! `rules/common/.../core` (`util/` = `core/util`), `rules` = `rules/common/.../rules`, `ktlint` = `rules/ktlint`
//! (the `KtlintRule` adapter, `.editorconfig` properties, the `*Check` modules, the provider).
//!
//! # Porting a rule
//! 1. `rules/<rule>.rs`: a unit struct implementing [`ComposeKtVisitor`]; messages are `pub const` strings with the
//!    `trimIndent()`ed text (lines joined by `\n`, no trailing newline).
//! 2. `ktlint/<rule>_check.rs`: `pub fn <rule>_check() -> KtlintRule` with upstream's id and properties
//!    (`ComposeProperty::String(&CUSTOM_MODIFIERS)`, ...), and `pub const CHECK = Some(<rule>_check)`. A check that
//!    overrides a visit method to gate it (`Material2Check`, `PreviewNamingCheck`, ...) is a struct holding the rule
//!    and implementing [`ComposeKtVisitor`] itself, like [`ktlint::preview_naming_check`]. No shared file changes:
//!    `rules/mod.rs`, `ktlint/mod.rs` and the provider already list every rule.
//! 3. Goldens: `cargo test -p ktrs-compose --release --test golden` (`GOLDEN_FILTER=<rule-dir>`), then
//!    `UPDATE_PASSING=1` to ratchet `tests/golden-passing.txt`.
//!
//! # Kotlin -> Rust
//! - PSI is `ktrs_ast::psi` over the mutable tree (its module docs have the conventions): `x is KtFoo` ->
//!   `KtFoo::is(ast, n)`, `x as? KtFoo` -> `KtFoo::cast(ast, n)`, getters drop `get` and take `ast`; expression
//!   results are bare `NodeId`s. `KtAnnotated`/`KtModifierListOwner`/`KtCallableDeclaration` receivers of the
//!   util extensions are bare nodes: `function.isComposable` -> `is_composable(ast, function.node())`.
//! - Visitor hooks get `ast: &mut Ast`; read through it freely (`&mut Ast` coerces to `&Ast`). Collect a Kotlin
//!   sequence into a `Vec` before reporting, unless a fix inside the loop edits the tree and the sequence is lazy
//!   upstream: then step a [`core::util::psi_elements::ChildrenByClass`].
//! - `emitter.report(e, msg)` -> `emitter.report(ast, e.node(), MSG, false)`; `report(e, msg, true).ifFix { .. }`
//!   -> `emitter.report(ast, e.node(), MSG, true).if_fix(|| { .. })` (the closure may mutate `ast`).
//! - `config.getSet(key, emptySet())` -> `config.get_set(key, &[])`; a Kotlin `Set<String>` is a `Vec<String>`
//!   without duplicates (insertion order). Regexes are `ktrs_lint::rules::internal::KotlinRegex` (Java syntax,
//!   `matches` = full match).
//! - `element.text` -> `ast.text(n)` or `x.text(ast)`; `startOffset` -> `ast.start_offset(n)`; `parents`,
//!   `parentsWithSelf`, `siblings(forward, withItself)` -> `ast.parents(n)`, `ast.parents_with_self(n)`,
//!   `ast.siblings_with_itself(n, forward, with_itself)`; `findAllChildrenByClass<T>()` ->
//!   `find_all_children::<T>(ast, n)` (breadth first over `getChildren()`, root included).
//! - Where ktlint 1.8.0 (Kotlin 2.2.21) and 2.0.0-ALPHA-4 (Kotlin 2.4.10) PSI differ, pass
//!   `config.embedded_kotlin()` (known: `quoteIfNeeded` on Latin-1 letters in `setName`, `getContextParameters`).
//!
//! Status, the parity gate and how to bump the pin: research/27-custom-rulesets-impl.md.

pub mod core;
pub mod ktlint;
pub mod rules;

pub use core::compose_kt_visitor::ComposeKtVisitor;
pub use ktlint::compose_rule_set_provider::compose_rule_providers;

/// The ported release.
pub const COMPOSE_RULES_VERSION: &str = "0.6.7";

/// (release, content fingerprint) of the ktlint JARs that run natively; the fingerprint is computed by
/// `ktrs-cli`'s `compose_jar.rs`. Per release: the published `ktlint-compose-<release>-all.jar`, and the Maven
/// artifacts `io.nlopez.compose.rules:ktlint` + `common-ktlint` merged (as the ktlint Gradle plugin passes them;
/// same bytecode, the `-all` JAR only relocates kotlin-compiler `psiUtil` calls: research/29).
pub const NATIVE_JARS: &[(&str, &str)] = &[
    ("0.6.7", "00de57c180c50a3fb36c345558d15e078bfe056bc158d79cc5e5ecbe6ee3bfdf"),
    ("0.6.7", "a9b34195dddeb441ab1eb2ef78e7080a6781d25654df05a9e1a569c88dc23878"),
];
