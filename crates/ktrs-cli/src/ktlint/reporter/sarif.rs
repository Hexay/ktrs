//! Port of `ktlint-cli-reporter-sarif` (`SarifReporter`): sarif4k's schema classes serialized as
//! kotlinx.serialization pretty-prints them (2-space indent, fields in declaration order, nulls omitted).

use std::path::{Path, PathBuf};

use crate::ktlint::console::Printer;
use crate::ktlint::reporter::{KTLINT_VERSION, KtlintCliError, ReporterV2, relative_to_or_self};

const SRCROOT: &str = "%SRCROOT%";

enum Json {
    Str(String),
    Num(usize),
    Obj(Vec<(&'static str, Json)>),
    Arr(Vec<Json>),
}

fn str(s: &str) -> Json {
    Json::Str(s.to_owned())
}

/// `String.sanitize()`: `/` separators and a trailing `/`.
pub fn sanitize(path: &str) -> String {
    let path = path.replace(std::path::MAIN_SEPARATOR, "/");
    if path.ends_with('/') { path } else { format!("{path}/") }
}

pub struct SarifReporter {
    out: Printer,
    user_home: Option<PathBuf>,
    results: Vec<Json>,
    working_directory: Option<PathBuf>,
}

impl SarifReporter {
    /// `user_home`: `System.getProperty("user.home")`, which upstream uses as the working directory.
    pub fn new(out: Printer, user_home: Option<PathBuf>) -> SarifReporter {
        SarifReporter { out, user_home, results: Vec::new(), working_directory: None }
    }
}

impl ReporterV2 for SarifReporter {
    fn before_all(&mut self) {
        self.working_directory = self.user_home.clone();
    }

    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError) {
        let artifact_location = match &self.working_directory {
            Some(working_directory) => Json::Obj(vec![
                ("uri", Json::Str(relative_to_or_self(Path::new(file), working_directory).to_string_lossy().into_owned())),
                ("uriBaseId", str(SRCROOT)),
            ]),
            None => Json::Obj(vec![("uri", str(file))]),
        };
        let region = Json::Obj(vec![
            ("startColumn", Json::Num(ktlint_cli_error.col)),
            ("startLine", Json::Num(ktlint_cli_error.line)),
        ]);
        let location = Json::Obj(vec![(
            "physicalLocation",
            Json::Obj(vec![("artifactLocation", artifact_location), ("region", region)]),
        )]);
        self.results.push(Json::Obj(vec![
            ("level", str("error")),
            ("locations", Json::Arr(vec![location])),
            ("message", Json::Obj(vec![("text", str(&ktlint_cli_error.detail))])),
            ("ruleId", str(&ktlint_cli_error.rule_id)),
        ]));
    }

    fn after_all(&mut self) {
        let version = KTLINT_VERSION;
        let driver = Json::Obj(vec![
            ("downloadUri", Json::Str(format!("https://github.com/ktlint/ktlint/releases/tag/{version}"))),
            ("fullName", str("ktlint")),
            ("informationUri", str("https://github.com/ktlint/ktlint/")),
            ("language", str("en")),
            ("name", str("ktlint")),
            ("organization", str("ktlint")),
            ("rules", Json::Arr(Vec::new())),
            ("semanticVersion", str(version)),
            ("version", str(version)),
        ]);
        let mut run = Vec::new();
        if let Some(working_directory) = &self.working_directory {
            let uri = format!("file://{}", sanitize(&working_directory.to_string_lossy()));
            run.push(("originalUriBaseIds", Json::Obj(vec![(SRCROOT, Json::Obj(vec![("uri", Json::Str(uri))]))])));
        }
        run.push(("results", Json::Arr(std::mem::take(&mut self.results))));
        run.push(("tool", Json::Obj(vec![("driver", driver)])));
        let schema = Json::Obj(vec![
            ("$schema", str("https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json")),
            ("version", str("2.1.0")),
            ("runs", Json::Arr(vec![Json::Obj(run)])),
        ]);
        let mut text = String::new();
        write_json(&mut text, &schema, 0);
        // `SarifSerializer.toJson` ends with a newline of its own (the jar prints a blank last line).
        text.push('\n');
        self.out.println(&text);
    }
}

/// kotlinx.serialization's pretty printer (`prettyPrintIndent = "  "`); lines end with `\n` on every OS.
fn write_json(out: &mut String, value: &Json, depth: usize) {
    let indent = |out: &mut String, depth: usize| (0..depth).for_each(|_| out.push_str("  "));
    match value {
        Json::Str(s) => write_quoted(out, s),
        Json::Num(n) => out.push_str(&n.to_string()),
        Json::Obj(fields) if fields.is_empty() => out.push_str("{}"),
        Json::Arr(items) if items.is_empty() => out.push_str("[]"),
        Json::Obj(fields) => {
            out.push('{');
            for (i, (key, field)) in fields.iter().enumerate() {
                out.push_str(if i == 0 { "\n" } else { ",\n" });
                indent(out, depth + 1);
                write_quoted(out, key);
                out.push_str(": ");
                write_json(out, field, depth + 1);
            }
            out.push('\n');
            indent(out, depth);
            out.push('}');
        }
        Json::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                out.push_str(if i == 0 { "\n" } else { ",\n" });
                indent(out, depth + 1);
                write_json(out, item, depth + 1);
            }
            out.push('\n');
            indent(out, depth);
            out.push(']');
        }
    }
}

fn write_quoted(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}
