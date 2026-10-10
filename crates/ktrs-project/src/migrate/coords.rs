//! What each drop-in replaces (README "Integrations"), and where notes send users.

pub(crate) const GRADLE_PLUGIN_ARTIFACT: &str = "io.github.hexay:ktrs-gradle-plugin";
/// The `io.github.hexay:ktrs` jar: Spotless steps (Gradle classpath, Maven plugin dependency).
pub(crate) const JAR_GROUP: &str = "io.github.hexay";
pub(crate) const JAR_ARTIFACT: &str = "ktrs";

pub(crate) const README_GRADLE: &str = "see README \"Integrations > Gradle\"";
pub(crate) const README_MAVEN: &str = "see README \"Integrations > Maven\"";
pub(crate) const README_DROP_IN: &str = "see README \"Drop-in for ktfmt and ktlint\"";

pub(crate) struct PluginSwap {
    pub old_id: &'static str,
    pub new_id: &'static str,
    /// The plugin's implementation artifact (`group:name`), as `buildscript`/convention builds depend on it.
    pub old_artifact: &'static str,
}

impl PluginSwap {
    /// Whether `matched` (an id, coordinates or catalog entry that matched one of [`GRADLE_PLUGINS`]) is this
    /// plugin's.
    pub(crate) fn names(&self, matched: &str) -> bool {
        let group = self.old_artifact.split(':').next().unwrap_or_default();
        matched.contains(self.old_id) || matched.contains(group)
    }
}

pub(crate) const KOTLINTER: PluginSwap = PluginSwap {
    old_id: "org.jmailen.kotlinter",
    new_id: "io.github.hexay.ktrs.kotlinter",
    old_artifact: "org.jmailen.gradle:kotlinter-gradle",
};

pub(crate) const GRADLE_PLUGINS: [PluginSwap; 3] = [
    PluginSwap {
        old_id: "com.ncorti.ktfmt.gradle",
        new_id: "io.github.hexay.ktrs",
        old_artifact: "com.ncorti.ktfmt.gradle:plugin",
    },
    PluginSwap {
        old_id: "org.jlleitschuh.gradle.ktlint",
        new_id: "io.github.hexay.ktrs.ktlint",
        old_artifact: "org.jlleitschuh.gradle:ktlint-gradle",
    },
    KOTLINTER,
];

pub(crate) fn plugin_by_old_id(id: &str) -> Option<&'static PluginSwap> {
    GRADLE_PLUGINS.iter().find(|p| p.old_id == id)
}

fn marker(id: &str) -> String {
    format!("{id}:{id}.gradle.plugin")
}

/// The replacement for a module of one of [`GRADLE_PLUGINS`] (`group:name`: the implementation artifact or
/// the plugin marker).
pub(crate) fn swap_module(module: &str) -> Option<String> {
    GRADLE_PLUGINS.iter().find_map(|p| {
        if module == p.old_artifact {
            Some(GRADLE_PLUGIN_ARTIFACT.to_string())
        } else if module == marker(p.old_id) {
            Some(marker(p.new_id))
        } else {
            None
        }
    })
}

/// `group:name[:version]` -> the swapped module and the version part, when `coords` is one of ours to swap.
pub(crate) fn swap_coords(coords: &str) -> Option<(String, Option<&str>)> {
    let mut parts = coords.splitn(3, ':');
    let (group, name) = (parts.next()?, parts.next()?);
    let new = swap_module(&format!("{group}:{name}"))?;
    Some((new, parts.next()))
}

pub(crate) const KTLINT_JAR: &str = "ktlint runs from its jar; point the task at the `ktlint` binary (same flags) by hand";
const KTFMT_JAR: &str = "ktfmt runs from its jar; point the task at the `ktfmt` binary (same flags) by hand";

/// Mentions of builds with no drop-in: (needle, what to do).
const NO_DROP_IN: [(&str, &str); 5] = [
    ("com.pinterest.ktlint.Main", KTLINT_JAR),
    ("com.pinterest.ktlint:ktlint-cli", KTLINT_JAR),
    ("com.pinterest:ktlint:", KTLINT_JAR),
    ("com.facebook.ktfmt.cli.Main", KTFMT_JAR),
    ("com.facebook:ktfmt:", KTFMT_JAR),
];

/// The first no-drop-in advice `text` mentions.
pub(crate) fn no_drop_in(text: &str) -> Option<&'static str> {
    // Not `ktlint-cli-ruleset-core` or `ktlint-cli-reporter-*`: a rule set's or reporter's API dependencies.
    let mentions = |needle: &str| text.match_indices(needle).any(|(i, m)| !text[i + m.len()..].starts_with('-'));
    NO_DROP_IN.iter().find(|(needle, _)| mentions(needle)).map(|(_, advice)| *advice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rule_set_api_dependency_is_not_ktlint_run_from_its_jar() {
        assert_eq!(no_drop_in(r#"compileOnly("com.pinterest.ktlint:ktlint-cli-ruleset-core:1.8.0")"#), None);
        assert_eq!(no_drop_in(r#"ktlint("com.pinterest.ktlint:ktlint-cli:1.8.0")"#), Some(KTLINT_JAR));
        assert_eq!(no_drop_in(r#"module = "com.pinterest.ktlint:ktlint-cli""#), Some(KTLINT_JAR));
    }
}
