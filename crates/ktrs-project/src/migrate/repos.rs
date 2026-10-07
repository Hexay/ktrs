//! The ktrs Maven repository the Gradle plugin resolves from until it is listed on the Plugin Portal
//! (README "Integrations > Gradle": `pluginManagement` in settings; the same line in a `repositories {}`
//! that resolves the plugin as a dependency).

use super::coords::PAGES_REPO;
use super::edits::{Edits, append_to_block, indent_unit, insert_at_top};
use super::scan::Script;

pub(crate) fn repo_line(groovy: bool) -> String {
    if groovy { format!("maven {{ url '{PAGES_REPO}' }}") } else { format!("maven(\"{PAGES_REPO}\")") }
}

/// Adds the repository to the settings' `pluginManagement { repositories {} }`, creating what's missing.
pub(crate) fn ensure_in_plugin_management(s: &Script, groovy: bool, edits: &mut Edits) {
    if s.text.contains(PAGES_REPO) {
        return;
    }
    let line = repo_line(groovy);
    let u = indent_unit(s.text);
    let repositories = format!("repositories {{\n{u}gradlePluginPortal()\n{u}{line}\n}}");
    match s.child_blocks(None, "pluginManagement").first() {
        None => {
            let indented = repositories.replace('\n', &format!("\n{u}"));
            insert_at_top(s, &format!("pluginManagement {{\n{u}{indented}\n}}\n"), edits);
        }
        Some(&pm) => match s.child_blocks(Some(pm), "repositories").first() {
            Some(&r) => append_to_block(s, r, &line, edits),
            None => append_to_block(s, pm, &repositories, edits),
        },
    }
}

/// Adds the repository to the `repositories {}` directly under `parent` (`None`: top level); `false` when
/// there is none.
pub(crate) fn ensure_in_repositories(s: &Script, parent: Option<usize>, groovy: bool, edits: &mut Edits) -> bool {
    if s.text.contains(PAGES_REPO) {
        return true;
    }
    let Some(&r) = s.child_blocks(parent, "repositories").first() else { return false };
    append_to_block(s, r, &repo_line(groovy), edits);
    true
}
