//! `toFilesURIList` and `KtlintServiceLoader.loadFromJarFile` for `-R` rule set and `artifact=` reporter
//! JARs. JVM code can't be loaded here: a JAR that declares the service exits like one that doesn't
//! (INVALID_RULESET_JAR), with a message saying why.

use std::path::Path;

use crate::ktlint::command_line::{Exit, ExitCode};
use crate::ktlint::file_utils::expand_tilde_to_full_path;
use crate::ktlint::jpath::JPath;
use crate::ktlint::logger::{KTLINT_COMMAND_LINE, KTLINT_SERVICE_LOADER, Level, Logger};

pub const RULE_SET_V2_PROVIDER: &str = "io.github.ktlint.core.cli.ruleset.core.api.RuleSetV2Provider";
pub const REPORTER_PROVIDER_V2: &str = "io.github.ktlint.core.cli.reporter.core.api.ReporterProviderV2";

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

/// The deprecated rule set interface upstream still loads (with a warning) before `RuleSetV2Provider`.
pub const RULE_SET_PROVIDER_V3: &str = "com.pinterest.ktlint.cli.ruleset.core.api.RuleSetProviderV3";

/// `loadFromJarFile(url, ..., ERROR_WHEN_REQUIRED_PROVIDER_IS_MISSING)` for a custom JAR, whose
/// `interface` (or one of `also_loadable`) implementation upstream would load.
pub fn load_from_jar_file(url_path: &str, interface: &str, also_loadable: &[&str], logger: &Logger) -> Exit {
    let file = if cfg!(windows) { url_path.trim_start_matches('/') } else { url_path };
    let declares_service = std::fs::read(Path::new(file)).is_ok_and(|bytes| {
        std::iter::once(&interface)
            .chain(also_loadable)
            .any(|i| contains(&bytes, format!("META-INF/services/{i}").as_bytes()))
    });
    if declares_service {
        logger.error(KTLINT_SERVICE_LOADER, || {
            format!("JAR file '{url_path}' implements interface '{interface}', but ktrs can not load JVM code")
        });
    } else if logger.is_enabled(Level::Debug) {
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
        logger.error(KTLINT_SERVICE_LOADER, || {
            format!(
                "JAR file '{url_path}' is missing a class implementing interface '{interface}' (run with '--log-level=debug' for more information)"
            )
        });
    }
    Exit::Code(ExitCode::InvalidRulesetJar)
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}
