//! detekt's light mode, natively: a port of detekt `v2.0.0-alpha.6` (pin: `tools/sync-detekt.sh`) over
//! [`ktrs_psi`]. Light mode is detekt's syntax-only analysis (`--analysis-mode light`, the CLI default); rules
//! that implement `RequiresAnalysisApi` are never ported. Plan, status and what is stubbed: research/33.
//!
//! # Layout (upstream module -> here)
//! - `detekt-api` -> [`api`]: `Config`, `Rule`, `Finding`, `Entity`, `Location`, `Issue`, signatures.
//! - `detekt-core` -> [`engine`] (rule descriptors, `Analyzer`, `@Suppress`) and [`config`] (YAML, composite,
//!   all-rules, default config).
//! - `detekt-psi-utils` -> [`psi`], `detekt-metrics` -> [`metrics`], `detekt-rules-*` -> [`rules`].
//! - Not upstream: [`kt_file`] (what a `KtFile` knows besides its tree), [`visitor`] (the sparse
//!   `DetektVisitor`), [`kotlin`] (stdlib/JDK behaviour).
//!
//! # Porting conventions
//! - One file per upstream file, one fn per upstream method, same order; a rule class is a struct with a
//!   [`api::RuleBase`], `impl Rule` (`rule_base!`) and its `visit*` overrides in [`detekt_visitor!`].
//!   `super.visitX(x)` is `kt_visitor_void::visit_x(self, x)`; an override that does not call it prunes the
//!   subtree, as upstream.
//! - `by config(default)` is a `OnceCell` field read through an accessor that calls
//!   [`api::config_property`] on first use; `String::toRegex` is [`kotlin::Regex::new`].
//! - An exception upstream is a panic with the same message; `Analyzer::analyze` lets it through and the
//!   caller turns it into the run's failure.
//! - Offsets are UTF-8 bytes; a `Location` holds detekt's 1-based line, UTF-16 column and UTF-16 offsets.
//! - `KtFile.name`, `containingFile`, `absolutePath()` and line lookups go through [`kt_file`].

pub mod api;
pub mod config;
pub mod engine;
pub mod kotlin;
pub mod kt_file;
pub mod metrics;
pub mod probe;
pub mod psi;
pub mod rules;
pub mod visitor;
