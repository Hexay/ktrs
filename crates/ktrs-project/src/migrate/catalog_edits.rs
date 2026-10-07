//! `gradle/libs.versions.toml`: ktfmt-gradle / ktlint-gradle entries under `[plugins]` (id) and `[libraries]`
//! (implementation artifact or marker) get ktrs's id or coordinates and version; a `version.ref` target is
//! rewritten when nothing else uses it, otherwise the entry gets its own `version`.

use std::ops::Range;

use super::Notes;
use super::coords::{plugin_by_old_id, swap_coords, swap_module};
use super::edits::Edits;
use crate::catalog::{normalize, strip_comment};

/// What the catalog swapped, for the builds that consume it.
#[derive(Default)]
pub(crate) struct Swapped {
    /// Accessors of swapped entries: `libs.plugins.ktfmt`, `libs.ktfmt.gradle.plugin`.
    pub accessors: Vec<String>,
    pub plugins: bool,
}

struct Entry {
    section: String,
    key: String,
    /// The value's span in the file.
    value: Range<usize>,
}

pub(crate) fn rewrite(text: &str, version: &str, edits: &mut Edits, notes: &mut Notes) -> Swapped {
    let entries = entries(text);
    let mut swapped = Swapped::default();
    for e in entries.iter().filter(|e| e.section == "plugins" || e.section == "libraries") {
        let value = &text[e.value.clone()];
        let at = e.value.start;
        let plugin = e.section == "plugins";
        let mut entry_edits = Edits::default();
        if let Some(s) = quoted(value) {
            let content = &value[s.clone()];
            let new = if plugin {
                content.split_once(':').and_then(|(id, _)| plugin_by_old_id(id)).map(|p| p.new_id.to_string())
            } else {
                swap_coords(content).map(|(module, _)| module)
            };
            let Some(new) = new else { continue };
            entry_edits.replace(at + s.start..at + s.end, format!("{new}:{version}"));
        } else {
            let matched = if plugin {
                swap_plugin_id(value, at, &mut entry_edits)
            } else {
                swap_library(value, at, &mut entry_edits)
            };
            if !matched {
                continue;
            }
            if let Err(why) = swap_version(text, &entries, value, at, version, &mut entry_edits) {
                notes.gradle(format!("[{}] {}: {why}; set it to \"{version}\" by hand", e.section, e.key));
                continue;
            }
        }
        edits.extend(entry_edits);
        swapped.plugins |= plugin;
        let alias = normalize(&e.key);
        swapped.accessors.push(if plugin { format!("libs.plugins.{alias}") } else { format!("libs.{alias}") });
    }
    swapped
}

fn swap_plugin_id(value: &str, at: usize, edits: &mut Edits) -> bool {
    let Some(r) = field(value, "id") else { return false };
    let Some(p) = plugin_by_old_id(&value[r.clone()]) else { return false };
    edits.replace(at + r.start..at + r.end, p.new_id);
    true
}

/// `module = "g:a"` or `group = "g", name = "a"`.
fn swap_library(value: &str, at: usize, edits: &mut Edits) -> bool {
    if let Some(r) = field(value, "module") {
        let Some(new) = swap_module(&value[r.clone()]) else { return false };
        edits.replace(at + r.start..at + r.end, new);
        return true;
    }
    let (Some(g), Some(n)) = (field(value, "group"), field(value, "name")) else { return false };
    let Some(new) = swap_module(&format!("{}:{}", &value[g.clone()], &value[n.clone()])) else { return false };
    let (group, name) = new.split_once(':').unwrap_or_default();
    edits.replace(at + g.start..at + g.end, group);
    edits.replace(at + n.start..at + n.end, name);
    true
}

fn swap_version(
    text: &str,
    entries: &[Entry],
    value: &str,
    at: usize,
    version: &str,
    edits: &mut Edits,
) -> Result<(), String> {
    if let Some(r) = field(value, "version") {
        edits.replace(at + r.start..at + r.end, version);
        return Ok(());
    }
    if let Some(r) = field(value, "version.ref") {
        let key = &value[r.clone()];
        let users = entries
            .iter()
            .filter(|e| field(&text[e.value.clone()], "version.ref").is_some_and(|k| text[e.value.clone()][k] == *key))
            .count();
        let target = entries.iter().find(|e| e.section == "versions" && e.key.trim_matches('"') == key);
        let target_str =
            target.and_then(|t| quoted(&text[t.value.clone()]).map(|q| t.value.start + q.start..t.value.start + q.end));
        match target_str {
            Some(t) if users == 1 => edits.replace(t, version),
            _ => {
                let start = value[..r.start].rfind("version.ref").unwrap_or(0);
                edits.replace(at + start..at + r.end + 1, format!("version = \"{version}\""));
            }
        }
        return Ok(());
    }
    if value.contains("version") {
        return Err("its version isn't a plain string".into());
    }
    Ok(())
}

/// Every `key = value` line with its section; values are single-line (inline tables are).
fn entries(text: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut section = String::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let body = strip_comment(line.trim_end_matches(['\n', '\r']));
        let trimmed = body.trim();
        if trimmed.starts_with('[') {
            section = trimmed.trim_matches(['[', ']']).trim().to_string();
            continue;
        }
        let Some(eq) = body.find('=') else { continue };
        let lead = body[eq + 1..].len() - body[eq + 1..].trim_start().len();
        let value_start = start + eq + 1 + lead;
        let value_end = start + body.trim_end().len();
        if value_start <= value_end {
            out.push(Entry {
                section: section.clone(),
                key: body[..eq].trim().to_string(),
                value: value_start..value_end,
            });
        }
    }
    out
}

/// The content range of a value that is a quoted string.
fn quoted(value: &str) -> Option<Range<usize>> {
    let q = value.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let end = value[1..].find(q)? + 1;
    Some(1..end)
}

/// The content range of `name = "x"` in an inline table.
fn field(table: &str, name: &str) -> Option<Range<usize>> {
    let mut from = 0;
    while let Some(pos) = table[from..].find(name).map(|p| p + from) {
        from = pos + name.len();
        let before_ok = table[..pos].trim_end().ends_with(['{', ',']);
        let rest = &table[from..];
        let after = rest.trim_start();
        if before_ok && after.starts_with('=') {
            let value = after[1..].trim_start();
            let value_at = table.len() - value.len();
            return quoted(value).map(|r| value_at + r.start..value_at + r.end);
        }
    }
    None
}
