//! The options of `KtlintCommandLine` and its subcommands (`Main.kt`), parsed the way Clikt does.

use std::path::Path;
use std::sync::LazyLock;

use ktrs_editorconfig::EnumValue;
use ktrs_lint::editorconfig::{CodeStyleValue, KtlintVersion};

use crate::ktlint::clikt::{Arity, Invocation, OptionSpec, expand_argument_files, parse_tokens};
use crate::ktlint::gradle;
use crate::ktlint::logger::Level;
use crate::ktlint::version::{KTLINT_VERSION_OPTION, release, resolve_ktlint_version};

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

static MAIN_OPTIONS: [OptionSpec; 21] = [
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
    OptionSpec { names: &[KTLINT_VERSION_OPTION], arity: Arity::Value, hidden: true },
    OptionSpec { names: &[gradle::EVENTS_OPTION], arity: Arity::Value, hidden: true },
    OptionSpec { names: &[gradle::RELATIVE_TO_OPTION], arity: Arity::Value, hidden: true },
    OptionSpec { names: &[gradle::EDITOR_CONFIG_OVERRIDE_OPTION], arity: Arity::Value, hidden: true },
];

/// 1.8 still declares `--code-style` (deprecated, an error when used) right after `--version`.
static MAIN_OPTIONS_1_8: LazyLock<Vec<OptionSpec>> = LazyLock::new(|| {
    let mut options = MAIN_OPTIONS.to_vec();
    options.insert(1, spec(&[CODE_STYLE_OPTION], Arity::Value));
    options
});

const CODE_STYLE_OPTION: &str = "--code-style";
const CODE_STYLE_CHOICES: &str = "(choose from android_studio, intellij_idea, ktlint_official)";

fn main_options(ktlint_version: KtlintVersion) -> &'static [OptionSpec] {
    if ktlint_version.is_1_8() { &MAIN_OPTIONS_1_8 } else { &MAIN_OPTIONS }
}

/// 1.8's help: the old documentation URL and the `--code-style` entry.
pub fn help_main(ktlint_version: KtlintVersion) -> String {
    if !ktlint_version.is_1_8() {
        return HELP_MAIN.to_owned();
    }
    HELP_MAIN.replace("(https://ktlint.github.io/ktlint/latest/)", "(https://pinterest.github.io/ktlint/latest/)").replace(
        "  -v, --version                  Show the version and exit\n",
        "  -v, --version                  Show the version and exit\n  \
         --code-style=(android_studio|intellij_idea|ktlint_official)\n                                 (deprecated)\n",
    )
}

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
    pub ktlint_version: KtlintVersion,
    /// ktrs-only (the Gradle plugin's): [`gradle`].
    pub gradle_events: Option<String>,
    pub relative_to: Option<String>,
    pub editor_config_overrides: Vec<String>,
    pub ktrs_lint: crate::ktlint::ktrs_only::KtrsLintOptions,
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
            ktlint_version: KtlintVersion::default(),
            gradle_events: None,
            relative_to: None,
            editor_config_overrides: Vec::new(),
            ktrs_lint: Default::default(),
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
    /// A plain `CliktError`: the message alone, for stderr, exit 1.
    Error(String),
}

pub fn parse_args(tokens: &[String], working_dir: &Path) -> Parsed {
    let tokens = match expand_argument_files(tokens, working_dir) {
        Ok(tokens) => tokens,
        Err(message) => return usage_error(USAGE_MAIN, message),
    };
    let ktlint_version = match resolve_ktlint_version(&tokens, working_dir) {
        Ok(ktlint_version) => ktlint_version,
        Err(message) => return usage_error(USAGE_MAIN, message),
    };
    let subcommands = [GENERATE_EDITOR_CONFIG, INSTALL_GIT_PRE_COMMIT_HOOK, INSTALL_GIT_PRE_PUSH_HOOK];
    let parsed = match parse_tokens(&tokens, main_options(ktlint_version), &subcommands) {
        Ok(parsed) => parsed,
        // Clikt runs eager options (help, version) before it reports a usage error.
        Err(_) if has_token(&tokens, &["--help", "-h"]) => return Parsed::Message(help_main(ktlint_version)),
        Err(_) if has_token(&tokens, &["--version", "-v"]) => return version(ktlint_version),
        Err(message) => return usage_error(USAGE_MAIN, with_subcommand_hint(message)),
    };
    let has = |name: &str| parsed.invocations.iter().any(|i| i.name == name);
    if has("--help") {
        return Parsed::Message(help_main(ktlint_version));
    }
    if has("--version") {
        return version(ktlint_version);
    }
    if let Some(code_style) = parsed.invocations.iter().rev().find(|i| i.name == CODE_STYLE_OPTION).and_then(|i| i.value.as_deref()) {
        return if code_style_value(code_style).is_some() {
            Parsed::Error(
                "Parameter '--code-style' is no longer valid. The code style should be defined as '.editorconfig' property \
                 'ktlint_code_style='"
                    .to_owned(),
            )
        } else {
            usage_error(USAGE_MAIN, format!("invalid value for {CODE_STYLE_OPTION}: invalid choice: {code_style}. {CODE_STYLE_CHOICES}"))
        };
    }
    let args = match to_ktlint_args(&parsed.invocations, parsed.arguments, ktlint_version) {
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

fn to_ktlint_args(invocations: &[Invocation], arguments: Vec<String>, ktlint_version: KtlintVersion) -> Result<KtlintArgs, String> {
    let mut args = KtlintArgs { arguments, ktlint_version, ..KtlintArgs::default() };
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
    args.gradle_events = last(gradle::EVENTS_OPTION);
    args.relative_to = last(gradle::RELATIVE_TO_OPTION);
    args.editor_config_overrides = all(gradle::EDITOR_CONFIG_OVERRIDE_OPTION).collect();
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
            code_style_value(&code_style).map(Subcommand::GenerateEditorConfig).ok_or_else(|| {
                usage_error(&usage, format!("invalid value for {CODE_STYLE_OPTION}: invalid choice: {code_style}. {CODE_STYLE_CHOICES}"))
            })
        }
        INSTALL_GIT_PRE_COMMIT_HOOK => Ok(Subcommand::InstallGitPreCommitHook),
        _ => Ok(Subcommand::InstallGitPrePushHook),
    }
}

/// Clikt's hint when an unknown option without suggestions is one of a subcommand's.
fn with_subcommand_hint(message: String) -> String {
    match message.strip_prefix("no such option ") {
        Some(name) if GENERATE_EDITOR_CONFIG_OPTIONS.iter().any(|o| o.names.contains(&name) && !o.names.contains(&"--help")) => {
            format!("{message}. hint: {GENERATE_EDITOR_CONFIG} has an option {name}")
        }
        _ => message,
    }
}

/// Clikt's `enum<CodeStyleValue>()`: the names, ignoring case.
fn code_style_value(value: &str) -> Option<CodeStyleValue> {
    CodeStyleValue::ENTRIES.iter().copied().find(|c| c.name().eq_ignore_ascii_case(value))
}

fn version(ktlint_version: KtlintVersion) -> Parsed {
    Parsed::Message(format!("ktlint version {}\n", release(ktlint_version)))
}

fn has_token(tokens: &[String], names: &[&str]) -> bool {
    tokens.iter().take_while(|t| *t != "--").any(|t| names.contains(&t.as_str()))
}

fn usage_error(usage: &str, message: String) -> Parsed {
    Parsed::UsageError { usage: usage.to_owned(), message }
}
