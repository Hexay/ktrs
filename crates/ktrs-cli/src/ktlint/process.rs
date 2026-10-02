//! `KtlintCommandLine.process`/`lint`/`format`: one file (or stdin) through the engine, to [`KtlintCliError`]s.

use std::sync::atomic::{AtomicBool, Ordering};

use ktrs_lint::editorconfig::KtlintVersion;
use ktrs_lint::{AutocorrectDecision, Code, KtLintException, KtLintRuleEngine, LintError};

use crate::ktlint::baseline::does_not_contain;
use crate::ktlint::command_line::{Exit, ExitCode};
use crate::ktlint::console::{Console, LINE_SEPARATOR};
use crate::ktlint::java_printf::java_printf;
use crate::ktlint::logger::{KTLINT_COMMAND_LINE, Logger};
use crate::ktlint::reporter::{KtlintCliError, Status};
use crate::ktlint::version::{package, repository};

pub struct Processor<'a> {
    pub engine: &'a KtLintRuleEngine,
    pub console: &'a Console,
    pub logger: &'a Logger,
    pub format: bool,
    pub ignore_autocorrect_failures: bool,
    pub force_lint_after_format: bool,
    pub ktlint_version: KtlintVersion,
    pub contains_unfixed_lint_errors: AtomicBool,
}

impl Processor<'_> {
    pub fn process(&self, code: &Code, baseline_lint_errors: &[KtlintCliError]) -> Result<Vec<KtlintCliError>, Exit> {
        if self.format { self.format(code, baseline_lint_errors) } else { self.lint(code, baseline_lint_errors) }
    }

    fn format(&self, code: &Code, baseline_lint_errors: &[KtlintCliError]) -> Result<Vec<KtlintCliError>, Exit> {
        let mut ktlint_cli_errors: Vec<KtlintCliError> = Vec::new();
        let result = self.engine.format(code, &mut |lint_error: &LintError| {
            if self.ignore_autocorrect_failures && !lint_error.can_be_auto_corrected {
                return AutocorrectDecision::NoAutocorrect;
            }
            let detail = if lint_error.can_be_auto_corrected {
                lint_error.detail.clone()
            } else {
                format!("{} (cannot be auto-corrected)", lint_error.detail)
            };
            let status = if lint_error.can_be_auto_corrected { Status::FormatIsAutocorrected } else { Status::LintCanNotBeAutocorrected };
            let ktlint_cli_error = KtlintCliError::new(lint_error.line, lint_error.col, lint_error.rule_id.value(), &detail, status);
            if !does_not_contain(baseline_lint_errors, &ktlint_cli_error) {
                return AutocorrectDecision::NoAutocorrect;
            }
            ktlint_cli_errors.push(ktlint_cli_error);
            if lint_error.can_be_auto_corrected {
                AutocorrectDecision::AllowAutocorrect
            } else {
                self.contains_unfixed_lint_errors.store(true, Ordering::SeqCst);
                AutocorrectDecision::NoAutocorrect
            }
        });
        match result {
            Ok(formatted_file_content) => {
                if !ktlint_cli_errors.is_empty() && code.content == formatted_file_content {
                    // Violations with opposite fixes can leave the file unchanged: treated as unfixable.
                    self.contains_unfixed_lint_errors.store(true, Ordering::SeqCst);
                }
                if self.force_lint_after_format && code.content != formatted_file_content {
                    self.lint_after_format(code, &formatted_file_content)?;
                }
                if code.is_std_in && self.ktlint_version.is_1_8() {
                    // 1.8 `printf`s the code: a `%` conversion in it fails (#3281); the exception escapes.
                    self.console.out(&java_printf(&formatted_file_content).map_err(Exit::Crash)?);
                } else if code.is_std_in {
                    self.console.out(&formatted_file_content);
                } else if code.content != formatted_file_content
                    && let Some(path) = &code.file_path
                    && let Err(e) = std::fs::write(path, &formatted_file_content)
                {
                    return Err(Exit::Crash(format!("java.io.FileNotFoundException: {} ({e})", path.display())));
                }
            }
            Err(e @ KtLintException::Parse(_)) if code.is_std_in && self.ktlint_version.is_1_8() => {
                return self.stdin_parse_exception_1_8(&e, code, |code| self.format(code, baseline_lint_errors));
            }
            Err(e) if code.is_std_in && !self.ktlint_version.is_1_8() => {
                return match e {
                    KtLintException::Parse(_) if code.script => {
                        self.log_stdin_script_parse_error(&e, code);
                        // The original code on stdout: an integration ignoring the exit code keeps its content.
                        self.console.out(&code.content);
                        Err(Exit::Code(ExitCode::ParseExceptionStdin))
                    }
                    KtLintException::Parse(_) => {
                        self.log_stdin_retry_as_script(&e, code);
                        self.format(&Code::from_snippet(&code.content, true), baseline_lint_errors)
                    }
                    _ => {
                        self.console.out(&code.content);
                        self.log_exception(&e, code)?;
                        Err(Exit::Code(ExitCode::ExceptionStdin))
                    }
                };
            }
            Err(e) => {
                if !self.ignore_autocorrect_failures {
                    ktlint_cli_errors.push(self.to_ktlint_cli_error(&e, code)?);
                    self.contains_unfixed_lint_errors.store(true, Ordering::SeqCst);
                }
            }
        }
        Ok(ktlint_cli_errors)
    }

    fn lint_after_format(&self, code: &Code, formatted_file_content: &str) -> Result<(), Exit> {
        match self.engine.lint(&Code::from_snippet(formatted_file_content, code.script), &mut |_| {}) {
            Err(KtLintException::Parse(e)) => {
                let file_path = code.file_path.as_ref().map_or("null".to_owned(), |p| p.display().to_string());
                self.logger.error(KTLINT_COMMAND_LINE, || {
                    format!(
                        "After formatting code in file '{file_path}' it cannot be successfully parsed anymore.{LINE_SEPARATOR}\
                         {}.rule.engine.api.KtLintParseException: {}:{} {}",
                        package(self.ktlint_version),
                        e.line,
                        e.col,
                        e.message
                    )
                });
                Err(Exit::Code(ExitCode::ParseExceptionAfterFormat))
            }
            _ => Ok(()),
        }
    }

    fn lint(&self, code: &Code, baseline_lint_errors: &[KtlintCliError]) -> Result<Vec<KtlintCliError>, Exit> {
        let mut ktlint_cli_errors: Vec<KtlintCliError> = Vec::new();
        let result = self.engine.lint(code, &mut |lint_error: &LintError| {
            let status =
                if lint_error.can_be_auto_corrected { Status::LintCanBeAutocorrected } else { Status::LintCanNotBeAutocorrected };
            let ktlint_cli_error =
                KtlintCliError::new(lint_error.line, lint_error.col, lint_error.rule_id.value(), &lint_error.detail, status);
            if does_not_contain(baseline_lint_errors, &ktlint_cli_error) && !self.ignore_autocorrect_failures {
                ktlint_cli_errors.push(ktlint_cli_error);
                self.contains_unfixed_lint_errors.store(true, Ordering::SeqCst);
            }
        });
        match result {
            Ok(()) => {}
            Err(e @ KtLintException::Parse(_)) if code.is_std_in && self.ktlint_version.is_1_8() => {
                return self.stdin_parse_exception_1_8(&e, code, |code| self.lint(code, baseline_lint_errors));
            }
            Err(e) if code.is_std_in && !self.ktlint_version.is_1_8() => {
                return match e {
                    KtLintException::Parse(_) if code.script => {
                        self.log_stdin_script_parse_error(&e, code);
                        Err(Exit::Code(ExitCode::ParseExceptionStdin))
                    }
                    KtLintException::Parse(_) => {
                        self.log_stdin_retry_as_script(&e, code);
                        self.lint(&Code::from_snippet(&code.content, true), baseline_lint_errors)
                    }
                    _ => {
                        self.log_exception(&e, code)?;
                        Err(Exit::Code(ExitCode::ExceptionStdin))
                    }
                };
            }
            Err(e) if !self.ignore_autocorrect_failures => {
                ktlint_cli_errors.push(self.to_ktlint_cli_error(&e, code)?);
                self.contains_unfixed_lint_errors.store(true, Ordering::SeqCst);
            }
            Err(_) => {}
        }
        Ok(ktlint_cli_errors)
    }

    /// 1.8: retry as a script, then log the error (still "as Kotlin") and report the parse error as a row;
    /// nothing goes to stdout and the exit code stays 0.
    fn stdin_parse_exception_1_8(
        &self,
        e: &KtLintException,
        code: &Code,
        retry_as_script: impl FnOnce(&Code) -> Result<Vec<KtlintCliError>, Exit>,
    ) -> Result<Vec<KtlintCliError>, Exit> {
        if !code.script {
            self.log_stdin_retry_as_script(e, code);
            return retry_as_script(&Code::from_snippet(&code.content, true));
        }
        let detail = self.detail(e, code);
        self.logger.error(KTLINT_COMMAND_LINE, || format!("Can not parse input from <stdin> as Kotlin, due to error below:\n    {detail}"));
        Ok(vec![self.to_ktlint_cli_error(e, code)?])
    }

    fn log_stdin_script_parse_error(&self, e: &KtLintException, code: &Code) {
        let detail = self.detail(e, code);
        self.logger.error(KTLINT_COMMAND_LINE, || {
            format!("Can not parse input from <stdin> as Kotlin script, due to error below:\n    {detail}")
        });
    }

    fn log_stdin_retry_as_script(&self, e: &KtLintException, code: &Code) {
        let detail = self.detail(e, code);
        self.logger.warn(KTLINT_COMMAND_LINE, || {
            format!(
                "Can not parse input from <stdin> as Kotlin, due to error below:\n    {detail}\nNow, trying to read the input as Kotlin Script."
            )
        });
    }

    fn detail(&self, e: &KtLintException, code: &Code) -> String {
        self.to_ktlint_cli_error(e, code).map(|e| e.detail).unwrap_or_default()
    }

    /// `logger.error(e) {}`: an empty message, then logback's rendering of the exception.
    fn log_exception(&self, e: &KtLintException, code: &Code) -> Result<(), Exit> {
        let exception = match e {
            KtLintException::Rule(_) => self.to_ktlint_cli_error(e, code)?.detail,
            _ => return Err(crash(e)),
        };
        self.logger.error(KTLINT_COMMAND_LINE, || format!("{LINE_SEPARATOR}{}", exception.trim_end()));
        Ok(())
    }

    /// `Exception.toKtlintCliError(code)`; other exceptions are rethrown (a crash).
    fn to_ktlint_cli_error(&self, e: &KtLintException, code: &Code) -> Result<KtlintCliError, Exit> {
        match e {
            KtLintException::Parse(p) => Ok(KtlintCliError::new(
                p.line,
                p.col,
                "",
                &format!("Not a valid Kotlin file ({})", format!("{}:{} {}", p.line, p.col, p.message).to_lowercase()),
                Status::KotlinParseException,
            )),
            KtLintException::Rule(r) => {
                let file = code.file_name_or_stdin();
                self.logger.debug(KTLINT_COMMAND_LINE, || {
                    format!("Internal Error ({}) in {file} at position '{}:{}", r.rule_id, r.line, r.col)
                });
                // The engine names 2.0's repository in the standard rules' `About`; 1.8's is pinterest's.
                let message = r.message.replace(repository(KtlintVersion::V2_0), repository(self.ktlint_version));
                let detail = format!(
                    "Internal Error (rule '{}') in {file} at position '{}:{}. Please create a ticket at \
                     {}/issues and provide the source code that triggered an error.\n\
                     {}.rule.engine.api.KtLintRuleException: {message}{LINE_SEPARATOR}Caused by: {}{LINE_SEPARATOR}",
                    r.rule_id,
                    r.line,
                    r.col,
                    repository(self.ktlint_version),
                    package(self.ktlint_version),
                    r.cause
                );
                Ok(KtlintCliError::new(r.line, r.col, "", &detail, Status::KtlintRuleEngineException))
            }
            KtLintException::EditorConfig(_) => Err(crash(e)),
            // Files are processed on a thread pool, whose `Future.get` wraps the exception.
            KtLintException::IllegalState(_) if !code.is_std_in => {
                Err(Exit::Crash(format!("java.util.concurrent.ExecutionException: {}", crash_text(e))))
            }
            KtLintException::IllegalState(_) => Err(crash(e)),
        }
    }
}

fn crash(e: &KtLintException) -> Exit {
    Exit::Crash(crash_text(e))
}

fn crash_text(e: &KtLintException) -> String {
    match e {
        KtLintException::EditorConfig(e) => format!("org.ec4j.core.parser.ParseException: {e}"),
        KtLintException::IllegalState(message) => format!("java.lang.IllegalStateException: {message}"),
        e => e.to_string(),
    }
}
