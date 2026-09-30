//! The options of `KtlintCommandLine` and its subcommands (`Main.kt`), parsed the way Clikt does.

use std::path::Path;

use ktrs_editorconfig::EnumValue;
use ktrs_lint::editorconfig::CodeStyleValue;

use crate::ktlint::clikt::{Arity, Invocation, OptionSpec, expand_argument_files, parse_tokens};
use crate::ktlint::logger::Level;
use crate::ktlint::reporter::KTLINT_VERSION;

pub const HELP_MAIN: &str = include_str!("help/help-main.txt");
const HELP_GENERATE_EDITOR_CONFIG: &str = include_str!("help/help-gen.txt");
const HELP_PRE_COMMIT: &str = include_str!("help/help-pre.txt");
const HELP_PRE_PUSH: &str = include_str!("help/help-push.txt");

pub const USAGE_MAIN: &str = "Usage: ktlint [<options>] [<arguments>]... <command> [<args>]...";
pub const GENERATE_EDITOR_CONFIG: &str = "generateEditorConfig";
pub const INSTALL_GIT_PRE_COMMIT_HOOK: &str = "installGitPreCommitHook";
pub const INSTALL_GIT_PRE_PUSH_HOOK: &str = "installGitPrePushHook";

const fn spec(names: &'static [&'static str], arity: Arity) -> OptionSpec {
    OptionSpec { names, arity, hidden: false }
}

static MAIN_OPTIONS: [OptionSpec; 17] = [
    spec(&["--version", "-v"], Arity::Flag),
    spec(&["--color"], Arity::Flag),
    spec(&["--color-name"], Arity::Value),
    spec(&["--format", "-F"], Arity::Flag),
    spec(&["--ignore-autocorrect-failures"], Arity::Flag),
    spec(&["--limit"], Arity::Value),
    spec(&["--relative"], Arity::Flag),
    spec(&["--reporter"], Arity::Value),
    spec(&["--ruleset", "-R"], Arity::Value),
    spec(&["--stdin"], Arity::Flag),
    spec(&["--stdin-path"], Arity::Value),
    spec(&["--patterns-from-stdin"], Arity::OptionalAttached),
    spec(&["--editorconfig"], Arity::Value),
    OptionSpec { names: &["--force-lint-after-format"], arity: Arity::Flag, hidden: true },
    spec(&["--baseline"], Arity::Value),
    spec(&["--log-level", "-l"], Arity::Value),
    spec(&["--help", "-h"], Arity::Flag),
];

static GENERATE_EDITOR_CONFIG_OPTIONS: [OptionSpec; 2] =
    [spec(&["--code-style"], Arity::Value), spec(&["--help", "-h"], Arity::Flag)];
static HELP_ONLY_OPTIONS: [OptionSpec; 1] = [spec(&["--help", "-h"], Arity::Flag)];

/// `KtlintCommandLine`'s options.
#[derive(Clone, Debug)]
pub struct KtlintArgs {
    pub color: bool,
    pub color_name: String,
    pub format: bool,
    pub ignore_autocorrect_failures: bool,
    pub limit: usize,
    pub relative: bool,
    pub reporter_configurations: Vec<String>,
    pub ruleset_jar_paths: Vec<String>,
    pub stdin: bool,
    pub stdin_path: Option<String>,
    pub patterns_from_stdin: Option<String>,
    pub editor_config_path: Option<String>,
    pub force_lint_after_format: bool,
    pub baseline_path: String,
    pub arguments: Vec<String>,
    pub min_log_level: Level,
}

impl Default for KtlintArgs {
    fn default() -> KtlintArgs {
        KtlintArgs {
            color: false,
            color_name: "DARK_GRAY".to_owned(),
            format: false,
            ignore_autocorrect_failures: false,
            limit: i32::MAX as usize,
            relative: false,
            reporter_configurations: Vec::new(),
            ruleset_jar_paths: Vec::new(),
            stdin: false,
            stdin_path: None,
            patterns_from_stdin: None,
            editor_config_path: None,
            force_lint_after_format: false,
            baseline_path: String::new(),
            arguments: Vec::new(),
            min_log_level: Level::Info,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subcommand {
    GenerateEditorConfig(CodeStyleValue),
    InstallGitPreCommitHook,
    InstallGitPrePushHook,
}

pub enum Parsed {
    Run(KtlintArgs, Option<Subcommand>),
    /// Help or version text for stdout, exit 0.
    Message(String),
    /// A usage error: the command's usage line and the message, for stderr, exit 1.
    UsageError { usage: String, message: String },
}

pub fn parse_args(tokens: &[String], working_dir: &Path) -> Parsed {
    let tokens = match expand_argument_files(tokens, working_dir) {
        Ok(tokens) => tokens,
        Err(message) => return usage_error(USAGE_MAIN, message),
    };
    let subcommands = [GENERATE_EDITOR_CONFIG, INSTALL_GIT_PRE_COMMIT_HOOK, INSTALL_GIT_PRE_PUSH_HOOK];
    let parsed = match parse_tokens(&tokens, &MAIN_OPTIONS, &subcommands) {
        Ok(parsed) => parsed,
        // Clikt runs eager options (help, version) before it reports a usage error.
        Err(_) if has_token(&tokens, &["--help", "-h"]) => return Parsed::Message(HELP_MAIN.to_owned()),
        Err(_) if has_token(&tokens, &["--version", "-v"]) => return version(),
        Err(message) => return usage_error(USAGE_MAIN, message),
    };
    let has = |name: &str| parsed.invocations.iter().any(|i| i.name == name);
    if has("--help") {
        return Parsed::Message(HELP_MAIN.to_owned());
    }
    if has("--version") {
        return version();
    }
    let args = match to_ktlint_args(&parsed.invocations, parsed.arguments) {
        Ok(args) => args,
        Err(message) => return usage_error(USAGE_MAIN, message),
    };
    let subcommand = match parsed.subcommand {
        None => None,
        Some((name, tokens)) => match parse_subcommand(name, &tokens) {
            Ok(subcommand) => Some(subcommand),
            Err(parsed) => return parsed,
        },
    };
    Parsed::Run(args, subcommand)
}

fn to_ktlint_args(invocations: &[Invocation], arguments: Vec<String>) -> Result<KtlintArgs, String> {
    let mut args = KtlintArgs { arguments, ..KtlintArgs::default() };
    let last = |name: &str| invocations.iter().rev().find(|i| i.name == name).and_then(|i| i.value.clone());
    let all = |name: &'static str| invocations.iter().filter(move |i| i.name == name).filter_map(|i| i.value.clone());
    let has = |name: &str| invocations.iter().any(|i| i.name == name);
    args.color = has("--color");
    if let Some(color_name) = last("--color-name") {
        args.color_name = color_name;
    }
    args.format = has("--format");
    args.ignore_autocorrect_failures = has("--ignore-autocorrect-failures");
    if let Some(limit) = last("--limit") {
        let limit: i32 =
            limit.parse().map_err(|_| format!("invalid value for --limit: {limit} is not a valid integer"))?;
        if limit <= 0 {
            return Err("invalid value for --limit: Value must be bigger than 0".to_owned());
        }
        args.limit = limit as usize;
    }
    args.relative = has("--relative");
    args.reporter_configurations = all("--reporter").collect();
    if let Some(rulesets) = last("--ruleset") {
        args.ruleset_jar_paths = rulesets.split(',').map(str::to_owned).collect();
    }
    args.stdin = has("--stdin");
    args.stdin_path = last("--stdin-path");
    args.patterns_from_stdin = last("--patterns-from-stdin");
    args.editor_config_path = last("--editorconfig");
    args.force_lint_after_format = has("--force-lint-after-format");
    if let Some(baseline) = last("--baseline") {
        args.baseline_path = baseline;
    }
    if let Some(level) = last("--log-level") {
        args.min_log_level = Level::parse(&level).map_err(|e| format!("invalid value for --log-level: {e}"))?;
    }
    Ok(args)
}

fn parse_subcommand(name: &'static str, tokens: &[String]) -> Result<Subcommand, Parsed> {
    let usage = format!("Usage: ktlint {name} [<options>]");
    let (options, help): (&'static [OptionSpec], &str) = match name {
        GENERATE_EDITOR_CONFIG => (&GENERATE_EDITOR_CONFIG_OPTIONS, HELP_GENERATE_EDITOR_CONFIG),
        INSTALL_GIT_PRE_COMMIT_HOOK => (&HELP_ONLY_OPTIONS, HELP_PRE_COMMIT),
        _ => (&HELP_ONLY_OPTIONS, HELP_PRE_PUSH),
    };
    let parsed = parse_tokens(tokens, options, &[]).map_err(|message| {
        if has_token(tokens, &["--help", "-h"]) { Parsed::Message(help.to_owned()) } else { usage_error(&usage, message) }
    })?;
    if parsed.invocations.iter().any(|i| i.name == "--help") {
        return Err(Parsed::Message(help.to_owned()));
    }
    if let Some(argument) = parsed.arguments.first() {
        return Err(usage_error(&usage, format!("got unexpected extra argument ({argument})")));
    }
    match name {
        GENERATE_EDITOR_CONFIG => {
            let code_style = parsed.invocations.iter().rev().find_map(|i| i.value.clone());
            let code_style = code_style.ok_or_else(|| usage_error(&usage, "missing option --code-style".to_owned()))?;
            CodeStyleValue::ENTRIES
                .iter()
                .copied()
                .find(|c| c.name().eq_ignore_ascii_case(&code_style))
                .map(Subcommand::GenerateEditorConfig)
                .ok_or_else(|| {
                    usage_error(
                        &usage,
                        format!(
                            "invalid value for --code-style: invalid choice: {code_style}. (choose from android_studio, \
                             intellij_idea, ktlint_official)"
                        ),
                    )
                })
        }
        INSTALL_GIT_PRE_COMMIT_HOOK => Ok(Subcommand::InstallGitPreCommitHook),
        _ => Ok(Subcommand::InstallGitPrePushHook),
    }
}

fn version() -> Parsed {
    Parsed::Message(format!("ktlint version {KTLINT_VERSION}\n"))
}

fn has_token(tokens: &[String], names: &[&str]) -> bool {
    tokens.iter().take_while(|t| *t != "--").any(|t| names.contains(&t.as_str()))
}

fn usage_error(usage: &str, message: String) -> Parsed {
    Parsed::UsageError { usage: usage.to_owned(), message }
}
