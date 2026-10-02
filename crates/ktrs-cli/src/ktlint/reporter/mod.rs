//! Ports of the built-in ktlint reporters (`ktlint-cli-reporter-*`) and of `ktlint-cli-reporter-core`.
//! A reporter is looked up by id ([`get_reporter`]), as ktlint's `ReporterProviderV2` service loader does.

pub mod baseline;
pub mod checkstyle;
pub mod format;
pub mod html;
pub mod java_map;
pub mod json;
pub mod plain;
pub mod plain_summary;
pub mod sarif;

use std::path::{Path, PathBuf};

use crate::ktlint::console::Printer;

/// `ktlintVersion(...)` of the 2.0 jar; the run's release: `version::release`.
pub const KTLINT_VERSION: &str = "2.0.0-ALPHA-4";

/// `KtlintCliError.Status`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    BaselineIgnored,
    LintCanNotBeAutocorrected,
    LintCanBeAutocorrected,
    FormatIsAutocorrected,
    KotlinParseException,
    KtlintRuleEngineException,
}

/// `KtlintCliError` (`@Poko`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KtlintCliError {
    pub line: usize,
    pub col: usize,
    pub rule_id: String,
    pub detail: String,
    pub status: Status,
}

impl KtlintCliError {
    pub fn new(line: usize, col: usize, rule_id: &str, detail: &str, status: Status) -> KtlintCliError {
        KtlintCliError { line, col, rule_id: rule_id.to_owned(), detail: detail.to_owned(), status }
    }
}

/// `ReporterV2`. Called from one thread, files in order.
pub trait ReporterV2: Send {
    fn before_all(&mut self) {}
    fn before(&mut self, _file: &str) {}
    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError);
    fn after(&mut self, _file: &str) {}
    fn after_all(&mut self) {}
}

/// A reporter's `opt` map (a `LinkedHashMap`: insertion order, later puts replace the value in place).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReporterOptions(pub Vec<(String, String)>);

impl ReporterOptions {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    pub fn put(&mut self, key: &str, value: &str) {
        match self.0.iter_mut().find(|(k, _)| k == key) {
            Some(entry) => entry.1 = value.to_owned(),
            None => self.0.push((key.to_owned(), value.to_owned())),
        }
    }

    /// Java's `AbstractMap.toString()`: `{k=v, k2=v2}`.
    pub fn to_java_string(&self) -> String {
        let entries: Vec<String> = self.0.iter().map(|(k, v)| format!("{k}={v}")).collect();
        format!("{{{}}}", entries.join(", "))
    }
}

/// What reporters read from the environment: `user.home` and the ktlint release (sarif) and the working
/// directory (baseline).
#[derive(Clone, Debug)]
pub struct ReporterEnvironment {
    pub user_home: Option<PathBuf>,
    pub working_dir: PathBuf,
    pub ktlint_release: &'static str,
}

/// The ids of the built-in `ReporterProviderV2`s.
pub const REPORTER_PROVIDER_IDS: [&str; 8] =
    ["baseline", "checkstyle", "format", "html", "json", "plain", "plain-summary", "sarif"];

/// `ReporterProviderV2.get(out, opt)` of the provider with `id`; `None` for an unknown id. `Err` is the
/// provider's exception (`IllegalArgumentException: ...`), which upstream does not catch.
pub fn get_reporter(
    id: &str,
    out: Printer,
    opt: &ReporterOptions,
    env: &ReporterEnvironment,
) -> Option<Result<Box<dyn ReporterV2>, String>> {
    let reporter: Result<Box<dyn ReporterV2>, String> = match id {
        "baseline" => Ok(Box::new(baseline::BaselineReporter::new(out, env.working_dir.clone()))),
        "checkstyle" => Ok(Box::new(checkstyle::CheckStyleReporter::new(out))),
        "format" => format::FormatReporterProvider::get(out, opt).map(|r| Box::new(r) as Box<dyn ReporterV2>),
        "html" => Ok(Box::new(html::HtmlReporter::new(out))),
        "json" => Ok(Box::new(json::JsonReporter::new(out))),
        "plain" => plain::PlainReporterProvider::get(out, opt).map(|r| Box::new(r) as Box<dyn ReporterV2>),
        "plain-summary" => Ok(Box::new(plain_summary::PlainSummaryReporter::new(out))),
        "sarif" => Ok(Box::new(sarif::SarifReporter::new(out, env.user_home.clone(), env.ktlint_release))),
        _ => return None,
    };
    Some(reporter)
}

/// `Color` (plain and format reporters).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    LightGray = 37,
    DarkGray = 90,
    LightRed = 91,
    LightGreen = 92,
    LightYellow = 93,
    LightBlue = 94,
    LightMagenta = 95,
    LightCyan = 96,
    White = 97,
}

impl Color {
    const ENTRIES: [(&'static str, Color); 16] = [
        ("BLACK", Color::Black),
        ("RED", Color::Red),
        ("GREEN", Color::Green),
        ("YELLOW", Color::Yellow),
        ("BLUE", Color::Blue),
        ("MAGENTA", Color::Magenta),
        ("CYAN", Color::Cyan),
        ("LIGHT_GRAY", Color::LightGray),
        ("DARK_GRAY", Color::DarkGray),
        ("LIGHT_RED", Color::LightRed),
        ("LIGHT_GREEN", Color::LightGreen),
        ("LIGHT_YELLOW", Color::LightYellow),
        ("LIGHT_BLUE", Color::LightBlue),
        ("LIGHT_MAGENTA", Color::LightMagenta),
        ("LIGHT_CYAN", Color::LightCyan),
        ("WHITE", Color::White),
    ];

    /// The providers' `getColor(opt["color_name"])`.
    pub fn from_option(name: Option<&str>) -> Result<Color, String> {
        Color::ENTRIES
            .iter()
            .find(|(n, _)| Some(*n) == name)
            .map(|(_, c)| *c)
            .ok_or_else(|| "java.lang.IllegalArgumentException: Invalid color parameter.".to_owned())
    }

    /// `String.color(foreground)`.
    pub fn paint(self, text: &str) -> String {
        format!("\u{1b}[{}m{text}\u{1b}[0m", self as u8)
    }
}

/// The providers' `String.emptyOrTrue()` over an optional option.
pub fn empty_or_true(value: Option<&str>) -> bool {
    matches!(value, Some("" | "true"))
}

/// Java's `File.separator` as used by `substringAfterLast(File.separator)` in the plain/format reporters.
pub const FILE_SEPARATOR: &str = if cfg!(windows) { "\\" } else { "/" };

/// `fileName.substring(0, len - name.length).colored() + name` of the plain and format reporters.
pub fn color_file_name(file_name: &str, colored: impl Fn(&str) -> String) -> String {
    let name = match file_name.rfind(FILE_SEPARATOR) {
        Some(i) => &file_name[i + FILE_SEPARATOR.len()..],
        None => file_name,
    };
    colored(&file_name[..file_name.len() - name.len()]) + name
}

/// `COUNT_DESC_AND_RULE_ID_ASC_COMPARATOR` over `(rule, count)` pairs, then the summary lines.
pub fn print_summary(out: &mut Printer, header: &str, counts: &[(String, u64)]) {
    let mut sorted = counts.to_vec();
    sorted.sort_by(|(ra, ca), (rb, cb)| cb.cmp(ca).then_with(|| java_compare(ra, rb)));
    out.println(header);
    for (rule, count) in sorted {
        out.println(&format!("  {rule}: {count}"));
    }
}

/// Java's `String.compareTo`: UTF-16 code unit order.
pub fn java_compare(a: &str, b: &str) -> std::cmp::Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

/// `ConcurrentHashMap.merge(key, 1, +)` over a small count list.
pub fn increment(counts: &mut Vec<(String, u64)>, key: &str) {
    match counts.iter_mut().rev().find(|(k, _)| k == key) {
        Some(entry) => entry.1 += 1,
        None => counts.push((key.to_owned(), 1)),
    }
}

/// `KtlintCliError.causedBy()` of the plain and plain-summary reporters.
pub fn caused_by(error: &KtlintCliError) -> String {
    match error.status {
        Status::KotlinParseException => "Not a valid Kotlin file".to_owned(),
        Status::KtlintRuleEngineException => "An internal error occurred in the Ktlint Rule Engine".to_owned(),
        _ if error.rule_id.is_empty() => "Unknown".to_owned(),
        _ => error.rule_id.clone(),
    }
}

/// `escapeXMLAttrValue()` of the checkstyle, baseline and html reporters.
pub fn escape_xml_attr_value(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// `acc.getOrPut(file) { ArrayList() }.add(error)` over a list keeping first-insertion order. Errors
/// arrive file by file, so the current file is the last entry.
pub fn accumulate(acc: &mut Vec<(String, Vec<KtlintCliError>)>, file: &str, error: &KtlintCliError) {
    match acc.iter_mut().rev().find(|(f, _)| f == file) {
        Some((_, errors)) => errors.push(error.clone()),
        None => acc.push((file.to_owned(), vec![error.clone()])),
    }
}

/// Kotlin's `Path.relativeToOrSelf(base)`: both lexically normalized, then `base.relativize(path)`;
/// `path` itself when the two are not both absolute (or both relative) with the same root.
pub fn relative_to_or_self(path: &Path, base: &Path) -> PathBuf {
    use std::path::Component;
    fn normalize(p: &Path) -> Vec<Component<'_>> {
        let mut parts: Vec<Component> = Vec::new();
        for c in p.components() {
            match c {
                Component::CurDir => {}
                Component::ParentDir if matches!(parts.last(), Some(Component::Normal(_))) => {
                    parts.pop();
                }
                c => parts.push(c),
            }
        }
        parts
    }
    fn root<'a>(parts: &[Component<'a>]) -> Vec<Component<'a>> {
        parts.iter().take_while(|c| matches!(c, Component::Prefix(_) | Component::RootDir)).cloned().collect()
    }
    let (p, b) = (normalize(path), normalize(base));
    if path.is_absolute() != base.is_absolute() || root(&p) != root(&b) {
        return path.to_path_buf();
    }
    let common = p.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let mut relative = PathBuf::new();
    for _ in common..b.len() {
        relative.push("..");
    }
    for c in &p[common..] {
        relative.push(c.as_os_str());
    }
    relative
}

/// `acc.entries.sortedBy { it.key }`.
pub fn sorted_by_file(acc: &[(String, Vec<KtlintCliError>)]) -> Vec<&(String, Vec<KtlintCliError>)> {
    let mut sorted: Vec<_> = acc.iter().collect();
    sorted.sort_by(|a, b| java_compare(&a.0, &b.0));
    sorted
}
