//! `KtlintCommandLine`'s per-run counters and its `report` (one file's errors to the reporter).

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::ktlint::args::KtlintArgs;
use crate::ktlint::process::Processor;
use crate::ktlint::reporter::{KtlintCliError, ReporterV2, Status};

/// The per-run state `lintOrFormat` shares with the file workers.
pub(crate) struct Run<'a> {
    pub(crate) processor: Processor<'a>,
    pub(crate) args: &'a KtlintArgs,
    pub(crate) file_number: AtomicUsize,
    pub(crate) error_number: AtomicUsize,
    pub(crate) advise_to_use_format: AtomicBool,
}

impl Run<'_> {
    pub(crate) fn report(&self, relative_route: &str, ktlint_cli_errors: &[KtlintCliError], reporter: &mut dyn ReporterV2) {
        self.file_number.fetch_add(1, Ordering::SeqCst);
        let err_list_limit = ktlint_cli_errors.len().min(self.args.limit.saturating_sub(self.error_number.load(Ordering::SeqCst)));
        self.error_number.fetch_add(err_list_limit, Ordering::SeqCst);
        if ktlint_cli_errors.iter().any(|e| e.status == Status::LintCanBeAutocorrected && !e.corrected) {
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
