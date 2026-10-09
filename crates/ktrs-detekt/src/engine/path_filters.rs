//! `detekt-utils/.../PathFilters.kt` and `PathMatchers.kt`, and the `Config.shouldAnalyzeFile` of
//! `detekt-core/.../util/ConfigExtensions.kt`.

use std::path::Path;

use ktrs_editorconfig::java_glob::PathMatcher;

use crate::api::config::{EXCLUDES_KEY, INCLUDES_KEY};
use crate::api::{Config, config_property};

pub struct PathFilters {
    includes: Option<Vec<PathMatcher>>,
    excludes: Option<Vec<PathMatcher>>,
}

impl PathFilters {
    /// Whether `path` (with `/` separators) is left out: not included, or excluded.
    pub fn is_ignored(&self, path: &str) -> bool {
        let is_included = self.includes.as_ref().is_none_or(|matchers| matchers.iter().any(|m| m.matches(path)));
        let is_excluded = self.excludes.as_ref().is_some_and(|matchers| matchers.iter().any(|m| m.matches(path)));
        !(is_included && !is_excluded)
    }

    pub fn of(includes: &[String], excludes: &[String]) -> Option<PathFilters> {
        if includes.is_empty() && excludes.is_empty() {
            return None;
        }
        Some(PathFilters { includes: parse(includes), excludes: parse(excludes) })
    }
}

fn parse(value: &[String]) -> Option<Vec<PathMatcher>> {
    if value.is_empty() { None } else { Some(value.iter().map(|pattern| path_matcher(pattern)).collect()) }
}

/// `pathMatcher(pattern)`: `FileSystems.getDefault().getPathMatcher("glob:...")`.
fn path_matcher(pattern: &str) -> PathMatcher {
    let glob = match pattern.split_once(':') {
        Some(("glob", glob)) => glob,
        Some(("regex", _)) => panic!(
            "java.lang.IllegalArgumentException: Only globbing patterns are supported as they are treated os-independently by the PathMatcher api."
        ),
        _ => pattern,
    };
    // Gotcha: the JDK's Windows file system matches globs case-insensitively.
    PathMatcher::new(glob, cfg!(windows)).unwrap_or_else(|e| panic!("java.util.regex.PatternSyntaxException: {e}"))
}

/// `Config.createPathFilters()`.
pub(crate) fn create_path_filters(config: &dyn Config) -> Option<PathFilters> {
    let includes = config_property::list(config, INCLUDES_KEY, &[]);
    let excludes = config_property::list(config, EXCLUDES_KEY, &[]);
    PathFilters::of(&includes, &excludes)
}

/// `filters == null || !filters.isIgnored(file, basePath)`: the path is matched as `./<relative to basePath>`.
pub(crate) fn should_analyze_file(filters: Option<&PathFilters>, file_path: &Path, base_path: &Path) -> bool {
    filters.is_none_or(|filters| !filters.is_ignored(&format!("./{}", relative_path(file_path, base_path))))
}

/// `absolutePath().relativeTo(basePath)` with `/` separators.
pub(crate) fn relative_path(file_path: &Path, base_path: &Path) -> String {
    let relative = file_path.strip_prefix(base_path).unwrap_or(file_path);
    relative.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/")
}
