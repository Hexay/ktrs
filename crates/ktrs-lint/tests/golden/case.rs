//! One golden case on disk (`testdata/ktlint/<rule-dir>/<case>.*`, written by
//! tools/ktlint-tests/extract/CaseRecorder.kt): `.input.kt|kts` (the extension is the script flag), `.options`,
//! and the real engine's results, each absent when empty: `.lint`/`.format` rows
//! `line:col\t<rule>\tauto|manual\t<detail>` (engine and callback order), `.expected.kt|kts` (formatted text
//! when it differs), `.error` (`lint|format\t<exception>`).

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Options {
    /// Rule under test first, then the additional rules.
    pub rules: Vec<String>,
    pub path: Option<String>,
    pub script: bool,
    /// Every override the test harness passes to the engine, forced ones included.
    pub editor_config: Vec<(String, String)>,
    /// The ktlint release the case was recorded on (`ktlint=1.8`; absent: the 2.0 line).
    pub ktlint: Option<String>,
}

#[derive(Clone)]
pub struct Case {
    pub name: String,
    pub input: String,
    pub options: Options,
    pub lint: Vec<String>,
    pub format: Vec<String>,
    pub expected: String,
    /// Stages (`lint`, `format`) where the engine threw, with the exception.
    pub errors: Vec<(String, String)>,
}

/// Case base paths (`<dir>/<case>`, without `.input.kt[s]`), sorted.
pub fn collect(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_into(dir, &mut out);
    out.sort();
    out
}

fn collect_into(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_into(&path, out);
        } else if let Some(base) = path.to_str().and_then(|p| p.strip_suffix(".input.kt").or_else(|| p.strip_suffix(".input.kts"))) {
            out.push(PathBuf::from(base));
        }
    }
}

pub fn read(base: &Path, suffix: &str) -> Option<String> {
    fs::read_to_string(format!("{}{suffix}", base.display())).ok()
}

pub fn rows(text: Option<String>) -> Vec<String> {
    text.map(|t| t.lines().map(str::to_owned).collect()).unwrap_or_default()
}

pub fn load(root: &Path, base: &Path) -> Case {
    let (script, input) = match read(base, ".input.kt") {
        Some(input) => (false, input),
        None => (true, read(base, ".input.kts").unwrap()),
    };
    let ext = if script { "kts" } else { "kt" };
    Case {
        name: base.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"),
        options: parse_options(&read(base, ".options").unwrap(), script),
        lint: rows(read(base, ".lint")),
        format: rows(read(base, ".format")),
        expected: read(base, &format!(".expected.{ext}")).unwrap_or_else(|| input.clone()),
        errors: rows(read(base, ".error"))
            .iter()
            .filter_map(|l| l.split_once('\t').map(|(stage, e)| (stage.to_owned(), e.to_owned())))
            .collect(),
        input,
    }
}

fn parse_options(text: &str, script: bool) -> Options {
    let mut options = Options { rules: Vec::new(), path: None, script, editor_config: Vec::new(), ktlint: None };
    for (key, value) in text.lines().filter_map(|l| l.split_once('=')) {
        match key {
            "rules" => options.rules = value.split(',').map(str::to_owned).collect(),
            "path" => options.path = Some(value.to_owned()),
            "ktlint" => options.ktlint = Some(value.to_owned()),
            "test" => {}
            _ => match key.strip_prefix("ec.") {
                Some(name) => options.editor_config.push((name.to_owned(), value.to_owned())),
                None => panic!("unknown option {key}"),
            },
        }
    }
    options
}
