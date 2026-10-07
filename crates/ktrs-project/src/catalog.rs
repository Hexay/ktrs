//! `gradle/libs.versions.toml`: the subset of TOML version catalogs use (sections, string and single-level
//! inline-table values).

use std::collections::HashMap;

#[derive(Default, Debug)]
pub(crate) struct Catalog {
    versions: HashMap<String, String>,
    /// `group:name[:version]`.
    libraries: HashMap<String, String>,
    plugins: HashMap<String, String>,
}

/// Aliases and accessors compare with `-`, `_` and `.` as one separator, case-insensitively.
pub(crate) fn normalize(alias: &str) -> String {
    alias.chars().map(|c| if c == '-' || c == '_' { '.' } else { c.to_ascii_lowercase() }).collect()
}

impl Catalog {
    pub(crate) fn parse(text: &str) -> Catalog {
        let mut catalog = Catalog::default();
        let mut raw: Vec<(String, String, String)> = Vec::new();
        let mut section = String::new();
        let mut lines = text.lines();
        while let Some(line) = lines.next() {
            let mut line = strip_comment(line).trim().to_string();
            if line.starts_with('[') {
                section = line.trim_matches(['[', ']']).trim().to_string();
                continue;
            }
            while line.matches('{').count() > line.matches('}').count() {
                let Some(next) = lines.next() else { break };
                line.push(' ');
                line.push_str(strip_comment(next).trim());
            }
            if let Some((key, value)) = line.split_once('=') {
                raw.push((section.clone(), normalize(key.trim().trim_matches('"')), value.trim().to_string()));
            }
        }
        for (_, key, value) in raw.iter().filter(|(s, ..)| s == "versions") {
            if let Some(v) = version_value(value) {
                catalog.versions.insert(key.clone(), v);
            }
        }
        for (section, key, value) in &raw {
            match section.as_str() {
                "libraries" => {
                    if let Some(coords) = catalog.library_value(value) {
                        catalog.libraries.insert(key.clone(), coords);
                    }
                }
                "plugins" => {
                    let id = match unquote(value) {
                        Some(s) => s.split(':').next().map(str::to_string),
                        None => table(value).remove("id"),
                    };
                    if let Some(id) = id {
                        catalog.plugins.insert(key.clone(), id);
                    }
                }
                _ => {}
            }
        }
        catalog
    }

    pub(crate) fn version(&self, alias: &str) -> Option<&str> {
        self.versions.get(&normalize(alias)).map(String::as_str)
    }

    pub(crate) fn library(&self, alias: &str) -> Option<&str> {
        self.libraries.get(&normalize(alias)).map(String::as_str)
    }

    pub(crate) fn plugin(&self, alias: &str) -> Option<&str> {
        self.plugins.get(&normalize(alias)).map(String::as_str)
    }

    fn library_value(&self, value: &str) -> Option<String> {
        if let Some(s) = unquote(value) {
            return Some(s);
        }
        let t = table(value);
        let module = t.get("module").cloned().or_else(|| Some(format!("{}:{}", t.get("group")?, t.get("name")?)))?;
        let version = t
            .get("version")
            .cloned()
            .or_else(|| t.get("version.ref").and_then(|r| self.version(r)).map(str::to_string));
        Some(match version {
            Some(v) => format!("{module}:{v}"),
            None => module,
        })
    }
}

fn version_value(value: &str) -> Option<String> {
    unquote(value).or_else(|| {
        let t = table(value);
        ["strictly", "require", "prefer"].iter().find_map(|k| t.get(*k).cloned())
    })
}

/// `{ a = "x", b.c = "y", version = { require = "z" } }` -> `a`, `b.c`, `version` (nested: its first value).
fn table(value: &str) -> HashMap<String, String> {
    let inner = value.trim().trim_start_matches('{').trim_end_matches('}');
    let mut out = HashMap::new();
    let mut depth = 0;
    let mut start = 0;
    let mut fields = Vec::new();
    for (i, c) in inner.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                fields.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    fields.push(&inner[start..]);
    for field in fields {
        if let Some((k, v)) = field.split_once('=') {
            let v = v.trim();
            let v = if v.starts_with('{') { version_value(v) } else { unquote(v) };
            if let Some(v) = v {
                out.insert(k.trim().to_string(), v);
            }
        }
    }
    out
}

fn unquote(value: &str) -> Option<String> {
    let v = value.trim();
    let q = v.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    Some(v[1..].split(q).next().unwrap_or_default().to_string())
}

fn strip_comment(line: &str) -> &str {
    let mut quote = None;
    for (i, c) in line.char_indices() {
        match (quote, c) {
            (None, '"' | '\'') => quote = Some(c),
            (Some(q), _) if c == q => quote = None,
            (None, '#') => return &line[..i],
            _ => {}
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_catalog() {
        let c = Catalog::parse(
            r#"
[versions]
ktlint = "1.8.0" # comment
ktfmt = { strictly = "0.64" }
[libraries]
compose-rules = { module = "io.nlopez.compose.rules:ktlint", version.ref = "ktlint" }
other = { group = "a", name = "b", version = "1" }
plain = "x:y:2"
[plugins]
ktlint-gradle = { id = "org.jlleitschuh.gradle.ktlint", version = "14.2.0" }
spotless = "com.diffplug.spotless:8.0.0"
"#,
        );
        assert_eq!(c.version("ktlint"), Some("1.8.0"));
        assert_eq!(c.version("ktfmt"), Some("0.64"));
        assert_eq!(c.library("compose.rules"), Some("io.nlopez.compose.rules:ktlint:1.8.0"));
        assert_eq!(c.library("other"), Some("a:b:1"));
        assert_eq!(c.library("plain"), Some("x:y:2"));
        assert_eq!(c.plugin("ktlint.gradle"), Some("org.jlleitschuh.gradle.ktlint"));
        assert_eq!(c.plugin("spotless"), Some("com.diffplug.spotless"));
    }
}
