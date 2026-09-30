//! Port of ktlint-cli `internal/KtlintCommandLine.kt` (and `Main.kt`): the `ktlint` command.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

use ktrs_lint::editorconfig::{RuleExecution, create_rule_execution_editor_config_property};
use ktrs_lint::rule_provider::{RuleV2Provider, property_types};
use ktrs_lint::rules::standard_rule_providers;
use ktrs_lint::{Code, EditorConfigDefaults, EditorConfigOverride, KtLintRuleEngine};

use crate::ktlint::args::{KtlintArgs, Parsed, Subcommand, USAGE_MAIN, parse_args};
use crate::ktlint::baseline::{Baseline, load_baseline};
use crate::ktlint::console::Console;
use crate::ktlint::file_utils::{expand_tilde_to_full_path, file_sequence, java_list, location};
use crate::ktlint::patterns::replace_with_patterns_from_stdin_or_default_patterns_when_empty;
use crate::ktlint::jar_providers::{RULE_SET_PROVIDER_V3, RULE_SET_V2_PROVIDER, load_from_jar_file, to_files_uri_list};
use crate::ktlint::jpath::JPath;
use crate::ktlint::logger::{
    EDITOR_CONFIG_DEFAULTS_LOADER, KTLINT_COMMAND_LINE, KTLINT_SERVICE_LOADER, LOAD_RULE_PROVIDERS, Logger,
};
use crate::ktlint::parallel::parallel;
use crate::ktlint::process::Processor;
use crate::ktlint::reporter::{KtlintCliError, ReporterEnvironment, ReporterV2, Status};
use crate::ktlint::reporter_aggregator::{Context, ReporterSettings, aggregated_reporter};
use crate::ktlint::subcommands;

/// `ExitCode`: values external integrations depend on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitCode {
    Ok = 0,
    HasUnfixedLintErrorsAfterFormat = 1,
    IoException = 2,
    ParseExceptionStdin = 3,
    ExceptionStdin = 4,
    FileNotFound = 5,
    InvalidRulesetJar = 6,
    InvalidReporterConfiguration = 7,
    ParseExceptionAfterFormat = 123,
}

/// How a run ends early: `exitKtLintProcess(code)`, a Clikt usage error, or an uncaught JVM exception.
#[derive(Debug)]
pub enum Exit {
    Code(ExitCode),
    Usage(String),
    Crash(String),
}

/// The `ktlint` command, run in `working_dir` (the JVM's `user.dir`).
pub struct KtlintCli {
    pub console: Console,
    pub working_dir: JPath,
    pub user_home: String,
}

impl KtlintCli {
    pub fn from_env() -> KtlintCli {
        let working_dir = std::env::current_dir().map(|d| JPath::from_path(&d)).unwrap_or_else(|_| JPath::parse(".").unwrap());
        let user_home = std::env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).unwrap_or_default();
        KtlintCli { console: Console::std(), working_dir, user_home }
    }

    pub fn run(&self, args: &[String]) -> i32 {
        let (args, subcommand) = match parse_args(args, &self.working_dir.to_path_buf()) {
            Parsed::Message(text) => {
                self.console.out(&text);
                return 0;
            }
            Parsed::UsageError { usage, message } => return self.usage_error(&usage, &message),
            Parsed::Run(args, subcommand) => (args, subcommand),
        };
        let logger = Logger::new(self.console.clone(), args.min_log_level);
        let result = match subcommand {
            None => self.lint_or_format(&args, &logger),
            Some(Subcommand::GenerateEditorConfig(code_style)) => {
                self.rule_providers(&args, &logger).and_then(|providers| subcommands::generate_editor_config(self, providers, code_style, &logger))
            }
            Some(Subcommand::InstallGitPreCommitHook) => subcommands::install_git_hook(self, subcommands::PRE_COMMIT),
            Some(Subcommand::InstallGitPrePushHook) => subcommands::install_git_hook(self, subcommands::PRE_PUSH),
        };
        self.exit_code(result, &logger)
    }

    /// Lints (or formats) with already parsed options: `ktrs lint`.
    pub fn run_lint(&self, args: &KtlintArgs) -> i32 {
        let logger = Logger::new(self.console.clone(), args.min_log_level);
        let result = self.lint_or_format(args, &logger);
        self.exit_code(result, &logger)
    }

    fn exit_code(&self, result: Result<(), Exit>, logger: &Logger) -> i32 {
        match result {
            Ok(()) => 0,
            Err(Exit::Code(code)) => {
                logger.debug(KTLINT_COMMAND_LINE, || format!("Exit ktlint with exit code: {}", code as i32));
                code as i32
            }
            Err(Exit::Usage(message)) => self.usage_error(USAGE_MAIN, &message),
            Err(Exit::Crash(exception)) => {
                self.console.println_err(&format!("Exception in thread \"main\" {exception}"));
                1
            }
        }
    }

    fn usage_error(&self, usage: &str, message: &str) -> i32 {
        self.console.err(&format!("{usage}\n\nError: {message}\n"));
        1
    }

    fn lint_or_format(&self, args: &KtlintArgs, logger: &Logger) -> Result<(), Exit> {
        if args.stdin && args.patterns_from_stdin.is_some() {
            return Err(Exit::Usage("option --stdin cannot be used with --patterns-from-stdin".to_owned()));
        }
        let patterns = replace_with_patterns_from_stdin_or_default_patterns_when_empty(args, &self.console, logger);
        let rule_providers = self.rule_providers(args, logger)?;
        let mut editor_config_override = EditorConfigOverride::empty();
        if args.stdin && args.stdin_path.as_deref().is_none_or(|p| p.trim().is_empty()) {
            logger.debug(KTLINT_COMMAND_LINE, || {
                "Add editor config override to disable 'filename' rule which can not be used in combination with reading from <stdin>"
                    .to_owned()
            });
            let filename = create_rule_execution_editor_config_property("standard:filename", RuleExecution::Disabled);
            editor_config_override = EditorConfigOverride::from(vec![(filename.into(), Some("disabled".to_owned()))]);
        }
        let start = Instant::now();
        let editor_config_defaults = self.editor_config_defaults(args, &rule_providers, logger)?;
        let engine = KtLintRuleEngine::with_editor_config(rule_providers, editor_config_defaults, editor_config_override);
        let baseline = if args.stdin || args.baseline_path.trim().is_empty() {
            Baseline::disabled()
        } else {
            load_baseline(&args.baseline_path, &self.working_dir.to_path_buf(), logger, &self.console)
        };
        let env = ReporterEnvironment { user_home: Some(self.user_home.clone().into()), working_dir: self.working_dir.to_path_buf() };
        let settings = ReporterSettings {
            reporter_configurations: &args.reporter_configurations,
            color: args.color,
            color_name: &args.color_name,
            stdin: args.stdin,
            format: args.format,
            relative: args.relative,
        };
        let cx = Context { console: &self.console, logger, working_dir: &self.working_dir, user_home: &self.user_home, env: &env };
        let mut reporter = aggregated_reporter(&baseline, &settings, &cx)?;
        let run = Run {
            processor: Processor {
                engine: &engine,
                console: &self.console,
                logger,
                format: args.format,
                ignore_autocorrect_failures: args.ignore_autocorrect_failures,
                force_lint_after_format: args.force_lint_after_format,
                contains_unfixed_lint_errors: AtomicBool::new(false),
            },
            args,
            file_number: AtomicUsize::new(0),
            error_number: AtomicUsize::new(0),
            advise_to_use_format: AtomicBool::new(false),
        };

        reporter.before_all();
        if args.stdin {
            self.lint_stdin(&run, &mut reporter)?;
        } else {
            self.lint_files(&run, &patterns, &baseline, &mut reporter, logger)?;
            if run.advise_to_use_format.load(Ordering::SeqCst) {
                if args.format {
                    logger.error(KTLINT_COMMAND_LINE, || {
                        "Format was not able to autocorrect all errors that theoretically can be autocorrected.".to_owned()
                    });
                } else {
                    logger.warn(KTLINT_COMMAND_LINE, || "Lint has found errors than can be autocorrected using 'ktlint --format'".to_owned());
                }
            }
        }
        reporter.after_all();
        drop(reporter);

        let (file_number, error_number) = (run.file_number.load(Ordering::SeqCst), run.error_number.load(Ordering::SeqCst));
        logger.debug(KTLINT_COMMAND_LINE, || {
            format!(
                "Finished processing in {}ms / {file_number} file(s) scanned / {error_number} error(s) found",
                start.elapsed().as_millis()
            )
        });
        if file_number == 0 {
            logger.warn(KTLINT_COMMAND_LINE, || format!("No files matched {}", java_list(&patterns)));
        }
        if run.processor.contains_unfixed_lint_errors.load(Ordering::SeqCst) {
            Err(Exit::Code(ExitCode::HasUnfixedLintErrorsAfterFormat))
        } else {
            Err(Exit::Code(ExitCode::Ok))
        }
    }

    /// `ruleProviders`: the standard rules; a `-R` JAR can't be loaded (see [`load_from_jar_file`]).
    pub(crate) fn rule_providers(&self, args: &KtlintArgs, logger: &Logger) -> Result<Vec<RuleV2Provider>, Exit> {
        let urls = to_files_uri_list(&args.ruleset_jar_paths, &self.working_dir, &self.user_home, logger)?;
        logger.debug(KTLINT_SERVICE_LOADER, || "Discovered RuleSetV2Provider with id 'standard' in ktlint JAR".to_owned());
        if let Some(url) = urls.first() {
            for message in [
                format!("Try loading ruleset provider of type 'RuleSetProviderV3' for file:{url}"),
                format!("Found 0 rule providers of type 'RuleSetProviderV3' for file:{url}"),
                format!("Try loading ruleset provider of type 'RuleSetV2Provider' for file:{url}"),
            ] {
                logger.debug(LOAD_RULE_PROVIDERS, || message);
            }
            return Err(load_from_jar_file(url, RULE_SET_V2_PROVIDER, &[RULE_SET_PROVIDER_V3], logger));
        }
        Ok(standard_rule_providers())
    }

    fn editor_config_defaults(&self, args: &KtlintArgs, rule_providers: &[RuleV2Provider], logger: &Logger) -> Result<EditorConfigDefaults, Exit> {
        let Some(path) = args.editor_config_path.as_ref().map(|p| expand_tilde_to_full_path(p, &self.user_home)) else {
            return Ok(EditorConfigDefaults::empty());
        };
        if path.trim().is_empty() {
            return Ok(EditorConfigDefaults::empty());
        }
        let resolved = self.working_dir.resolve(&path).map(|p| p.to_path_buf()).unwrap_or_else(|| path.clone().into());
        let file = if resolved.is_dir() { resolved.join(".editorconfig") } else { resolved.clone() };
        if !file.exists() {
            logger.warn(EDITOR_CONFIG_DEFAULTS_LOADER, || format!("File or directory '{path}' is not found. Can not load '.editorconfig' properties"));
            return Ok(EditorConfigDefaults::empty());
        }
        EditorConfigDefaults::load(Some(&resolved), &property_types(rule_providers))
            .map_err(|e| Exit::Crash(format!("org.ec4j.core.parser.ParseException: {e}")))
    }

    fn lint_files(&self, run: &Run, patterns: &[String], baseline: &Baseline, reporter: &mut dyn ReporterV2, logger: &Logger) -> Result<(), Exit> {
        let root_dir = self.working_dir.normalize();
        let files = file_sequence(patterns, &root_dir, &self.user_home, logger)
            .map_err(|e| Exit::Crash(format!("java.util.regex.PatternSyntaxException: {e}")))?;
        let failure: Mutex<Option<Exit>> = Mutex::new(None);
        let reporter = Mutex::new(reporter);
        parallel(
            &files,
            || run.error_number.load(Ordering::SeqCst) >= run.args.limit || failure.lock().unwrap().is_some(),
            |file| {
                // Baseline stores the lint violations as relative path to work dir
                let baseline_lint_errors = baseline.lint_errors_per_file.get(&location(file, true, &self.working_dir));
                let code = Code::from_file(&file.to_path_buf())
                    .map_err(|e| Exit::Crash(format!("java.io.IOException: {e}")))?;
                run.processor.process(&code, baseline_lint_errors.map_or(&[][..], Vec::as_slice))
            },
            |file, result| match result {
                Ok(errors) => run.report(&location(file, run.args.relative, &self.working_dir), &errors, &mut **reporter.lock().unwrap()),
                Err(exit) => {
                    failure.lock().unwrap().get_or_insert(exit);
                }
            },
        );
        match failure.into_inner().unwrap() {
            Some(exit) => Err(exit),
            None => Ok(()),
        }
    }

    fn lint_stdin(&self, run: &Run, reporter: &mut dyn ReporterV2) -> Result<(), Exit> {
        let content = String::from_utf8_lossy(&self.console.read_stdin()).into_owned();
        let code = match &run.args.stdin_path {
            Some(path) => {
                let path = expand_tilde_to_full_path(path, &self.user_home);
                let virtual_path = self.working_dir.resolve(&path).map(|p| p.to_path_buf()).unwrap_or_else(|| path.into());
                Code::from_snippet_with_path(&content, Some(&virtual_path))
            }
            None => Code::from_snippet_with_path(&content, None),
        };
        let errors = run.processor.process(&code, &[])?;
        run.report("<stdin>", &errors, reporter);
        Ok(())
    }
}

/// The per-run state `lintOrFormat` shares with the file workers.
struct Run<'a> {
    processor: Processor<'a>,
    args: &'a KtlintArgs,
    file_number: AtomicUsize,
    error_number: AtomicUsize,
    advise_to_use_format: AtomicBool,
}

impl Run<'_> {
    fn report(&self, relative_route: &str, ktlint_cli_errors: &[KtlintCliError], reporter: &mut dyn ReporterV2) {
        self.file_number.fetch_add(1, Ordering::SeqCst);
        let err_list_limit = ktlint_cli_errors.len().min(self.args.limit.saturating_sub(self.error_number.load(Ordering::SeqCst)));
        self.error_number.fetch_add(err_list_limit, Ordering::SeqCst);
        if ktlint_cli_errors.iter().any(|e| e.status == Status::LintCanBeAutocorrected) {
            self.advise_to_use_format.store(true, Ordering::SeqCst);
        }
        reporter.before(relative_route);
        ktlint_cli_errors
            .iter()
            .take(err_list_limit)
            .filter(|e| !(self.args.ignore_autocorrect_failures && e.status == Status::LintCanNotBeAutocorrected))
            .for_each(|e| reporter.on_lint_error(relative_route, e));
        reporter.after(relative_route);
    }
}
