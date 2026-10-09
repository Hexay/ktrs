//! `ktrs lsp`: a language server (stdio, LSP 3.17) for ktlint diagnostics and ktfmt or ktlint formatting, meant
//! to run next to a Kotlin language server such as kotlin-lsp. Design notes: research/32-ide-lsp.md.
//!
//! # Features
//! - Full text sync. Diagnostics are pushed (`textDocument/publishDiagnostics`) on open, change and save; changes
//!   that arrive while a lint runs are coalesced, so only a document's latest text is linted.
//! - A diagnostic is a ktlint lint error: `code` = rule id, `source` = `ktlint`, severity Warning (Error for
//!   one that can't be autocorrected with `ktlint.unfixableAsError`, as the IntelliJ ktlint plugin's option),
//!   `data` = `{"autocorrectable": bool}`. Its range is the token at ktlint's line and column, zero-width when
//!   there is none. An unparsable file gets no diagnostics.
//! - `textDocument/formatting` with the effective formatter: ktfmt, or ktlint's `--format` (fix every
//!   autocorrectable error). The result is one edit covering the changed lines.
//! - `textDocument/rangeFormatting`, ktfmt only (ktlint: `null` and a log line): ktfmt's partial formatting
//!   (`--offset`/`--length`). The statements or members the range touches are pretty-printed whole; an empty
//!   range means the one at the cursor. As in ktfmt, the whole-file cleanups still run afterwards (unused and
//!   unsorted imports, redundant semicolons, managed trailing commas, `trimIndent` strings), so the edit can
//!   reach outside the range.
//! - `textDocument/codeAction`, for the ktlint errors in the range: `Fix <rule>` (quickfix, only that error),
//!   `Suppress <rule> on this line` / `in this file` (quickfix, ktlint's `insertSuppression`), and
//!   `Fix all autocorrectable ktlint violations` (`source.fixAll.ktlint`).
//! - `workspace/didChangeWatchedFiles` (registered dynamically when the client allows): `.editorconfig` changes
//!   reload ktlint's cache, build-file changes invalidate the detected project config; open documents are re-linted.
//!
//! # Configuration
//! Per file: the build files' configuration (`ktrs_project::detect`), then the editor settings below, then the
//! fallback when neither says anything: ktlint 1.8 diagnostics, no formatting. The effective configuration and
//! where it came from are logged (`window/logMessage`) once per build root, and again when it changes.
//!
//! Settings come from `initializationOptions` and `workspace/didChangeConfiguration` (which replaces them), as
//! a JSON object, either bare or under a `ktrs` key; every key is optional:
//!
//! | key | values | meaning |
//! |---|---|---|
//! | `format.tool` | `auto`, `ktfmt`, `ktlint`, `none` | the formatter; `auto`: the build's, else none |
//! | `ktfmt.style` | `meta`, `google`, `kotlinlang` | ktfmt's style (default: the build's, else `meta`) |
//! | `ktfmt.maxWidth`, `ktfmt.blockIndent`, `ktfmt.continuationIndent` | positive integers | over the style's |
//! | `ktfmt.removeUnusedImports`, `ktfmt.manageTrailingCommas` | booleans | over the style's |
//! | `ktfmt.editorconfig` | boolean, default `false` | apply `.editorconfig` like ktfmt's `--editorconfig` |
//! | `ktlint.enable` | `auto`, `true`, `false` | diagnostics; `auto`: when the build uses ktlint or detects nothing |
//! | `ktlint.version` | `1.8`, `2.0` | the ktlint release imitated (default: the build's, else 1.8) |
//! | `ktlint.android`, `ktlint.experimental` | booleans | `ktlint_code_style = android_studio`, `ktlint_experimental = enabled` |
//! | `ktlint.editorconfigOverrides` | object of `.editorconfig` properties | win over every `.editorconfig` |
//! | `ktlint.ruleSets` | array of JAR paths | only compose-rules runs (natively); others are skipped with a warning |
//! | `ktlint.unfixableAsError` | boolean, default `false` | severity Error for non-autocorrectable errors |
//!
//! Editor settings win over the build's values key by key; `ktlint.ruleSets` and `ktlint.editorconfigOverrides`
//! replace the build's lists.

mod code_actions;
mod config;
mod diagnostics;
mod documents;
mod formatting;
mod ktlint;
mod server;
mod settings;
mod text;

pub use documents::{path_to_uri, uri_to_path};
pub use server::serve;
pub use text::apply_edits;

/// Runs the server on stdin/stdout until the client exits; returns the process exit code.
pub fn run_stdio() -> i32 {
    let (connection, io_threads) = lsp_server::Connection::stdio();
    let result = serve(connection);
    let joined = io_threads.join();
    match result.and(joined.map_err(|e| e.to_string())) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("ktrs lsp: {e}");
            1
        }
    }
}
