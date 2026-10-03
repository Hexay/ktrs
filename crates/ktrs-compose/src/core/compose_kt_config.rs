//! Port of `core/ComposeKtConfig.kt`.

use ktrs_ast::psi::EmbeddedKotlin;

/// `ComposeKtConfig`. A Kotlin `Set<String>` is a `Vec<String>` without duplicates, in insertion order.
pub trait ComposeKtConfig {
    fn get_int(&self, key: &str, default: i32) -> i32;
    fn get_string(&self, key: &str, default: Option<&str>) -> Option<String>;
    fn get_list(&self, key: &str, default: &[String]) -> Vec<String>;
    fn get_set(&self, key: &str, default: &[String]) -> Vec<String>;
    fn get_boolean(&self, key: &str, default: bool) -> bool;

    /// ktrs: the Kotlin compiler embedded by the ktlint release this run imitates, for the few PSI calls whose
    /// result differs between them (`ktrs_ast::psi::EmbeddedKotlin`).
    fn embedded_kotlin(&self) -> EmbeddedKotlin {
        EmbeddedKotlin::default()
    }
}
