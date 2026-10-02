//! `toFilesURIList`, `ruleProviders` and `KtlintServiceLoader.loadFromJarFile` for `-R` rule set and
//! `artifact=` reporter JARs. A rule set JAR that declares a loadable provider can't run here: the whole run
//! is handed to the ktlint jar instead ([`jvm_rule_set_jar`], `ktlint_jar.rs`). A JAR that declares none fails
//! natively, with upstream's message.

use std::path::Path;

use ktrs_lint::editorconfig::KtlintVersion;
use crate::ktlint::version::package;
use ktrs_lint::rule_provider::RuleV2Provider;
use ktrs_lint::rules::standard_rule_providers;

use crate::ktlint::KtlintCli;
use crate::ktlint::args::KtlintArgs;
use crate::ktlint::command_line::{Exit, ExitCode};
use crate::ktlint::file_utils::expand_tilde_to_full_path;
use crate::ktlint::jpath::JPath;
use crate::ktlint::logger::{KTLINT_COMMAND_LINE, KTLINT_SERVICE_LOADER, LOAD_RULE_PROVIDERS, Level, Logger};

pub const RULE_SET_V2_PROVIDER: &str = "io.github.ktlint.core.cli.ruleset.core.api.RuleSetV2Provider";

/// The deprecated rule set interface upstream still loads (with a warning) before `RuleSetV2Provider`.
pub const RULE_SET_PROVIDER_V3: &str = "com.pinterest.ktlint.cli.ruleset.core.api.RuleSetProviderV3";

/// The rule set interfaces `ktlint_version` loads (1.8 only knows `RuleSetProviderV3`).
fn rule_set_interfaces(ktlint_version: KtlintVersion) -> &'static [&'static str] {
    if ktlint_version.is_1_8() { &[RULE_SET_PROVIDER_V3] } else { &[RULE_SET_V2_PROVIDER, RULE_SET_PROVIDER_V3] }
}

/// The first JAR of the run that only the JVM can run: a `-R` JAR declaring a rule set provider, or an
/// `artifact=` JAR declaring a reporter provider, that `args.ktlint_version` loads.
pub fn jvm_only_jar(args: &KtlintArgs, working_dir: &JPath, user_home: &str) -> Option<String> {
    let version = args.ktlint_version;
    let reporter_interface = format!("{}.cli.reporter.core.api.ReporterProviderV2", package(version));
    let artifacts: Vec<String> = args
        .reporter_configurations
        .iter()
        .filter_map(|c| c.split(',').rev().find_map(|e| e.strip_prefix("artifact=")).map(str::to_owned))
        .collect();
    first_declaring(&args.ruleset_jar_paths, rule_set_interfaces(version), working_dir, user_home)
        .or_else(|| first_declaring(&artifacts, &[&reporter_interface], working_dir, user_home))
}

/// `None` when any path is missing too, so that the native run reports it as upstream does.
fn first_declaring(paths: &[String], interfaces: &[&str], working_dir: &JPath, user_home: &str) -> Option<String> {
    let files: Option<Vec<_>> = paths
        .iter()
        .map(|path| working_dir.resolve(&expand_tilde_to_full_path(path, user_home)).map(|f| f.to_path_buf()).filter(|f| f.exists()))
        .collect();
    files?.iter().zip(paths).find(|(file, _)| declares_any_service(file, interfaces)).map(|(_, path)| path.clone())
}

/// `List<String>.toFilesURIList()`: each path must exist (else FILE_NOT_FOUND); the URL paths, distinct.
pub fn to_files_uri_list(paths: &[String], working_dir: &JPath, user_home: &str, logger: &Logger) -> Result<Vec<String>, Exit> {
    let mut urls: Vec<String> = Vec::new();
    for path in paths {
        let file = working_dir.resolve(&expand_tilde_to_full_path(path, user_home));
        if !file.as_ref().is_some_and(|f| f.to_path_buf().exists()) {
            logger.error(KTLINT_COMMAND_LINE, || format!("File '{path}' does not exist"));
            return Err(Exit::Code(ExitCode::FileNotFound));
        }
        let url_path = url_path(&file.unwrap());
        if !urls.contains(&url_path) {
            urls.push(url_path);
        }
    }
    Ok(urls)
}

/// `File.toURI().toURL().path`: absolute, `/`-rooted (`/C:/x` on Windows).
fn url_path(file: &JPath) -> String {
    let path = file.normalize().to_string();
    if path.starts_with('/') { path } else { format!("/{path}") }
}

impl KtlintCli {
    /// `ruleProviders`: the standard rules. A `-R` JAR that gets here declares no loadable provider (the
    /// others were handed to the ktlint jar), so it fails as upstream does.
    pub(crate) fn rule_providers(&self, args: &KtlintArgs, logger: &Logger) -> Result<Vec<RuleV2Provider>, Exit> {
        let urls = to_files_uri_list(&args.ruleset_jar_paths, &self.working_dir, &self.user_home, logger)?;
        if args.ktlint_version.is_1_8() {
            logger.debug(KTLINT_SERVICE_LOADER, || "Discovered RuleSetProviderV3 with id 'standard' in ktlint JAR".to_owned());
            if let Some(url) = urls.first() {
                return Err(load_from_jar_file(url, RULE_SET_PROVIDER_V3, logger));
            }
            return Ok(standard_rule_providers());
        }
        logger.debug(KTLINT_SERVICE_LOADER, || "Discovered RuleSetV2Provider with id 'standard' in ktlint JAR".to_owned());
        if let Some(url) = urls.first() {
            for message in [
                format!("Try loading ruleset provider of type 'RuleSetProviderV3' for file:{url}"),
                format!("Found 0 rule providers of type 'RuleSetProviderV3' for file:{url}"),
                format!("Try loading ruleset provider of type 'RuleSetV2Provider' for file:{url}"),
            ] {
                logger.debug(LOAD_RULE_PROVIDERS, || message);
            }
            return Err(load_from_jar_file(url, RULE_SET_V2_PROVIDER, logger));
        }
        Ok(standard_rule_providers())
    }
}

/// `loadFromJarFile(url, ..., ERROR_WHEN_REQUIRED_PROVIDER_IS_MISSING)` for a custom JAR without an
/// `interface` implementation (one with it was handed to the ktlint jar).
pub fn load_from_jar_file(url_path: &str, interface: &str, logger: &Logger) -> Exit {
    if logger.is_enabled(Level::Debug) {
        logger.error(KTLINT_SERVICE_LOADER, || {
            [
                format!("JAR file '{url_path}' is missing a class implementing interface '{interface}'"),
                "    KtLint uses a ServiceLoader to dynamically load classes from JAR files specified at the command line of KtLint.".to_owned(),
                "    The JAR file below does not contain an implementation of the interface.".to_owned(),
                format!("        Interface: {interface}"),
                format!("        JAR file : {url_path}"),
                "    Check following:".to_owned(),
                "      - Does the jar contain an implementation of the interface above?".to_owned(),
                format!("      - Does the jar contain a resource file with name '{interface}'?"),
                "      - Is the resource file located in directory \"src/main/resources/META-INF/services\"?".to_owned(),
                "      - Does the resource file contain the fully qualified class name of the class implementing the interface above?".to_owned(),
            ]
            .join("\n")
        });
    } else {
        let hint = if logger.ktlint_version().is_1_8() { "run in debug mode" } else { "run with '--log-level=debug'" };
        logger.error(KTLINT_SERVICE_LOADER, || {
            format!("JAR file '{url_path}' is missing a class implementing interface '{interface}' ({hint} for more information)")
        });
    }
    Exit::Code(ExitCode::InvalidRulesetJar)
}

/// Whether the JAR has a `META-INF/services/<interface>` entry (zip entry names are stored uncompressed).
fn declares_any_service(jar: &Path, interfaces: &[&str]) -> bool {
    std::fs::read(jar).is_ok_and(|bytes| interfaces.iter().any(|i| contains(&bytes, format!("META-INF/services/{i}").as_bytes())))
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}
