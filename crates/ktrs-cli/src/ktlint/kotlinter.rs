//! Hidden, ktrs-only option of the kotlinter drop-in (`io.github.hexay.ktrs.kotlinter`, research/34). kotlinter runs
//! ktlint's engine in a Gradle worker with its own reporting; `--ktrs-kotlinter-events=<file>` makes a CLI run do
//! what its worker actions do and writes what they log to `<file>` (the plugin prints it):
//!
//! | run | effect |
//! |---|---|
//! | lint | `LintWorkerAction`: every `--reporter` inside a [`SortedThreadSafeReporterWrapper`], with the file's path relative to the working directory (the project's), absolute for `sarif` ([`reporter_path_for`]); no default reporter, no baseline, no `--limit` |
//! | `--format` | `FormatWorkerAction`: `engine.format` with a callback that allows every autocorrect; no reporters |
//!
//! The events file: the header of [`gradle`]'s, then per file, in argument order, `file\t<absolute path>`, an
//! `error` line ([`gradle::error_event`]) per callback of the engine (lint: sorted, distinct; format: emit order,
//! once per pass) and `formatted\t<path>` when the text changed. What the engine throws is an `error` line with the
//! exception's status and `Throwable.message` as detail; like the worker, the run stops there: later files are
//! neither reported nor rewritten, and the reporters never get `afterAll` (their files stay empty).
//!
//! A run handed to the ktlint jar writes the events file with the `json` reporter instead (`hand_off_args`).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use ktrs_lint::editorconfig::KtlintVersion;
use ktrs_lint::{AutocorrectDecision, Code, KtLintException, LintError};

use crate::ktlint::baseline::Baseline;
use crate::ktlint::command_line::{Exit, ExitCode, KtlintCli};
use crate::ktlint::file_utils::{file_sequence, location};
use crate::ktlint::gradle::{self, native_separators};
use crate::ktlint::parallel::parallel;
use crate::ktlint::process::Processor;
use crate::ktlint::reporter::{KtlintCliError, ReporterV2, Status, java_compare};
use crate::ktlint::reporter_aggregator::{Context, ReporterSettings, configured_reporters};
use crate::ktlint::run::Run;
use crate::ktlint::version::repository;

pub const EVENTS_OPTION: &str = "--ktrs-kotlinter-events";

/// What a worker action gets from the engine for one file.
pub(crate) struct Outcome {
    errors: Vec<KtlintCliError>,
    /// The formatted text, when it differs from the file's.
    formatted: Option<String>,
    exception: Option<KtlintCliError>,
}

impl KtlintCli {
    /// `LintWorkerAction.execute` / `FormatWorkerAction.execute` over the run's files; files are processed in
    /// parallel, their results applied in order.
    pub(crate) fn kotlinter(
        &self,
        run: &Run,
        patterns: &[String],
        settings: &ReporterSettings,
        cx: &Context,
        events: &str,
    ) -> Result<(), Exit> {
        let files = file_sequence(patterns, &self.working_dir.normalize(), &self.user_home, cx.logger)
            .map_err(|e| Exit::Crash(format!("java.util.regex.PatternSyntaxException: {e}")))?;
        let mut reporters: Vec<(bool, SortedThreadSafeReporterWrapper)> = Vec::new();
        if !run.args.format && !settings.reporter_configurations.is_empty() {
            for (id, reporter) in configured_reporters(&Baseline::disabled(), settings, cx)? {
                reporters.push((id == "sarif", SortedThreadSafeReporterWrapper::new(reporter)));
            }
        }
        let path = cx.working_dir.resolve(events).map(|p| p.to_path_buf()).unwrap_or_else(|| events.into());
        let file = File::create(&path).map_err(|e| Exit::Crash(format!("java.io.FileNotFoundException: {events} ({e})")))?;
        let mut out = BufWriter::new(file);
        let _ = writeln!(out, "{}", gradle::EVENTS_HEADER);
        reporters.iter_mut().for_each(|(_, r)| r.before_all());

        let failure: Mutex<Option<Exit>> = Mutex::new(None);
        let aborted = AtomicBool::new(false);
        parallel(
            &files,
            || aborted.load(Ordering::SeqCst) || failure.lock().unwrap().is_some(),
            |file| {
                let code = Code::from_file(&file.to_path_buf()).map_err(|e| Exit::Crash(format!("java.io.IOException: {e}")))?;
                run.processor.kotlinter_process(&code)
            },
            |file, result| {
                if aborted.load(Ordering::SeqCst) {
                    return;
                }
                let outcome = match result {
                    Ok(outcome) => outcome,
                    Err(exit) => {
                        failure.lock().unwrap().get_or_insert(exit);
                        return;
                    }
                };
                let absolute = native_separators(&location(file, false, &self.working_dir));
                let relative = native_separators(&location(file, true, &self.working_dir));
                let _ = writeln!(out, "file\t{absolute}");
                reporters.iter_mut().for_each(|(_, r)| r.before(&relative));
                for e in &outcome.errors {
                    let _ = writeln!(out, "{}", gradle::error_event(&absolute, e));
                    reporters.iter_mut().for_each(|(sarif, r)| r.on_lint_error(reporter_path_for(*sarif, &absolute, &relative), e));
                }
                if let Some(text) = &outcome.formatted {
                    if let Err(e) = std::fs::write(file.to_path_buf(), text) {
                        failure.lock().unwrap().get_or_insert(Exit::Crash(format!("java.io.FileNotFoundException: {absolute} ({e})")));
                        return;
                    }
                    let _ = writeln!(out, "formatted\t{absolute}");
                }
                if let Some(e) = &outcome.exception {
                    let _ = writeln!(out, "{}", gradle::error_event(&absolute, e));
                    aborted.store(true, Ordering::SeqCst);
                    return;
                }
                reporters.iter_mut().for_each(|(_, r)| r.after(&relative));
            },
        );
        let _ = out.flush();
        if let Some(exit) = failure.into_inner().unwrap() {
            return Err(exit);
        }
        if !aborted.load(Ordering::SeqCst) {
            reporters.iter_mut().for_each(|(_, r)| r.after_all());
        }
        drop(reporters);
        if run.processor.contains_unfixed_lint_errors.load(Ordering::SeqCst) {
            Err(Exit::Code(ExitCode::HasUnfixedLintErrorsAfterFormat))
        } else {
            Err(Exit::Code(ExitCode::Ok))
        }
    }
}

impl Processor<'_> {
    /// The engine call of kotlinter's worker actions. The run fails (`hasError`) on any lint error, and on a
    /// format error that can't be autocorrected.
    pub(crate) fn kotlinter_process(&self, code: &Code) -> Result<Outcome, Exit> {
        let mut errors: Vec<KtlintCliError> = Vec::new();
        let mut on_error = |e: &LintError| {
            let status = if e.can_be_auto_corrected { Status::LintCanBeAutocorrected } else { Status::LintCanNotBeAutocorrected };
            if !(self.format && e.can_be_auto_corrected) {
                self.contains_unfixed_lint_errors.store(true, Ordering::SeqCst);
            }
            errors.push(KtlintCliError::new(e.line, e.col, e.rule_id.value(), &e.detail, status));
        };
        let result = if self.format {
            let formatted = self.engine.format(code, &mut |e| {
                on_error(e);
                AutocorrectDecision::AllowAutocorrect
            });
            formatted.map(|text| (text != code.content).then_some(text))
        } else {
            self.engine.lint(code, &mut on_error).map(|()| None)
        };
        match result {
            Ok(formatted) => Ok(Outcome { errors, formatted, exception: None }),
            Err(e) => {
                self.contains_unfixed_lint_errors.store(true, Ordering::SeqCst);
                let exception = self.kotlinter_exception(&e, code)?;
                Ok(Outcome { errors, formatted: None, exception: Some(exception) })
            }
        }
    }

    /// What the engine threw, with its `Throwable.message` (which `workerErrorMessage` shows) as detail; the
    /// CLI's crashes stay crashes.
    fn kotlinter_exception(&self, e: &KtLintException, code: &Code) -> Result<KtlintCliError, Exit> {
        match e {
            KtLintException::Parse(p) => {
                let message = format!("{}:{} {}", p.line, p.col, p.message);
                Ok(KtlintCliError::new(p.line, p.col, "", &message, Status::KotlinParseException))
            }
            KtLintException::Rule(r) => {
                let message = r.message.replace(repository(KtlintVersion::V2_0), repository(self.ktlint_version));
                Ok(KtlintCliError::new(r.line, r.col, "", &message, Status::KtlintRuleEngineException))
            }
            e => self.to_ktlint_cli_error(e, code),
        }
    }
}

/// `reporterPathFor`: some reporters want relative paths, some (`sarif`) want absolute.
fn reporter_path_for<'a>(sarif: bool, absolute: &'a str, relative: &'a str) -> &'a str {
    if sarif { absolute } else { relative }
}

/// Port of kotlinter's `support/SortedThreadSafeReporterWrapper.kt`: the calls to the wrapped reporter are delayed
/// until `afterAll`, files then come in `String` order.
pub struct SortedThreadSafeReporterWrapper {
    wrapped: Box<dyn ReporterV2>,
    calls_to_before: HashSet<String>,
    // A `ConcurrentSkipListSet` ordered by (line, col) only: a second error at a position is dropped.
    lint_error_reports: HashMap<String, BTreeMap<(usize, usize), KtlintCliError>>,
    calls_to_after: HashSet<String>,
}

impl SortedThreadSafeReporterWrapper {
    pub fn new(wrapped: Box<dyn ReporterV2>) -> SortedThreadSafeReporterWrapper {
        SortedThreadSafeReporterWrapper {
            wrapped,
            calls_to_before: HashSet::new(),
            lint_error_reports: HashMap::new(),
            calls_to_after: HashSet::new(),
        }
    }
}

impl ReporterV2 for SortedThreadSafeReporterWrapper {
    fn before_all(&mut self) {
        self.wrapped.before_all();
    }

    fn before(&mut self, file: &str) {
        self.calls_to_before.insert(file.to_owned());
    }

    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError) {
        self.lint_error_reports
            .entry(file.to_owned())
            .or_default()
            .entry((ktlint_cli_error.line, ktlint_cli_error.col))
            .or_insert_with(|| ktlint_cli_error.clone());
    }

    fn after(&mut self, file: &str) {
        self.calls_to_after.insert(file.to_owned());
    }

    fn after_all(&mut self) {
        let mut file_names: Vec<&String> =
            self.calls_to_before.iter().chain(self.lint_error_reports.keys()).chain(&self.calls_to_after).collect();
        file_names.sort_by(|a, b| java_compare(a, b));
        file_names.dedup();
        for file_name in file_names {
            if self.calls_to_before.contains(file_name) {
                self.wrapped.before(file_name);
            }
            if let Some(lint_error_reports) = self.lint_error_reports.get(file_name) {
                lint_error_reports.values().for_each(|e| self.wrapped.on_lint_error(file_name, e));
            }
            if self.calls_to_after.contains(file_name) {
                self.wrapped.after(file_name);
            }
        }
        self.wrapped.after_all();
    }
}

#[cfg(test)]
mod tests;
