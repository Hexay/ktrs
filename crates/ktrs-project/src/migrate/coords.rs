//! What each drop-in replaces (README "Integrations"), and where notes send users.

pub(crate) const PAGES_REPO: &str = "https://hexay.github.io/ktrs/maven";
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

pub(crate) const GRADLE_PLUGINS: [PluginSwap; 2] = [
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
];

pub(crate) fn plugin_by_old_id(id: &str) -> Option<&'static PluginSwap> {
    GRADLE_PLUGINS.iter().find(|p| p.old_id == id)
}

fn marker(id: &str) -> String {
    format!("{id}:{id}.gradle.plugin")
}

/// The replacement for a ktfmt-gradle / ktlint-gradle module (`group:name`: the implementation artifact or
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

const KOTLINTER: &str =
    "kotlinter runs ktlint inside the JVM and has no ktrs drop-in; replace it with `ktrs lint` or the `ktlint` binary by hand";
pub(crate) const KTLINT_JAR: &str = "ktlint runs from its jar; point the task at the `ktlint` binary (same flags) by hand";
const KTFMT_JAR: &str = "ktfmt runs from its jar; point the task at the `ktfmt` binary (same flags) by hand";

/// Mentions of builds with no drop-in: (needle, what to do).
const NO_DROP_IN: [(&str, &str); 6] = [
    ("org.jmailen.kotlinter", KOTLINTER),
    ("com.pinterest.ktlint.Main", KTLINT_JAR),
    ("com.pinterest.ktlint:ktlint-cli", KTLINT_JAR),
    ("com.pinterest:ktlint:", KTLINT_JAR),
    ("com.facebook.ktfmt.cli.Main", KTFMT_JAR),
    ("com.facebook:ktfmt:", KTFMT_JAR),
];

/// The first no-drop-in advice `text` mentions.
pub(crate) fn no_drop_in(text: &str) -> Option<&'static str> {
    NO_DROP_IN.iter().find(|(needle, _)| text.contains(needle)).map(|(_, advice)| *advice)
}
