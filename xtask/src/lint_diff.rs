//! `cargo lint-diff [ktlint_official|intellij_idea|android_studio] [--experimental] [--oracle DIR] [--counts] [--corpus DIR]`:
//! lints (and, when the oracle ran exactly the ported rules, formats) every corpus file with ktrs-lint and diffs
//! against the real ktlint engine, pre-built on the JVM by (background, testbox):
//!   KTLINT_CODE_STYLE=<style> tools/ktlint-oracle/ktlint-probe.sh corpus target/ktlint-oracle/<style> [--rules <ported>]
//! Lint rows are compared per file as a multiset over the ported rules (lint is per-rule independent, so an
//! all-rules oracle filters down). An oracle run with exactly the ported rules (`--rules`) also compares the
//! engine's suppression rule, format rows (emit order) and formatted bytes. Where the oracle's format threw
//! (failed.tsv), ours must throw the same exception; lint rows are still compared there. Both sides run on a staged LF copy under
//! one root `.editorconfig` (target/lint-diff/<style>); `--experimental` adds `ktlint_experimental = enabled` there,
//! matching an oracle built with `KTLINT_EXPERIMENTAL=enabled` into target/ktlint-oracle/<style>-experimental. Mismatches go to target/lint-diff-<style>.txt; `--counts`
//! prints the oracle's per-rule violation counts instead.

use std::collections::{HashMap, HashSet};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fs, thread};

use ktrs_lint::rules::standard_rule_providers;
use ktrs_lint::{AutocorrectDecision, Code, KtLintException, KtLintRuleEngine, LintError};

use crate::corpus_diff::collect;
use crate::lint_oracle::{self, Oracle, Row, SUPPRESSION_RULE, rule_of};
use crate::lint_report::{FileResult, report};

const STYLES: [&str; 3] = ["ktlint_official", "intellij_idea", "android_studio"];

struct Args {
    style: String,
    experimental: bool,
    oracle: PathBuf,
    corpus: PathBuf,
    counts: bool,
}

impl Args {
    /// Names the default oracle dir, the staged copy and the report: `<style>[-experimental]`.
    fn run_name(&self) -> String {
        if self.experimental { format!("{}-experimental", self.style) } else { self.style.clone() }
    }
}

fn parse_args(root: &Path, args: &[String]) -> Result<Args, String> {
    let mut parsed =
        Args { style: STYLES[0].to_owned(), experimental: false, oracle: PathBuf::new(), corpus: root.join("corpus"), counts: false };
    let mut oracle = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--oracle" => oracle = rest.next().map(PathBuf::from),
            "--corpus" => parsed.corpus = rest.next().map(PathBuf::from).ok_or("--corpus needs a dir")?,
            "--counts" => parsed.counts = true,
            "--experimental" => parsed.experimental = true,
            s if STYLES.contains(&s) => parsed.style = s.to_owned(),
            s => {
                return Err(format!(
                    "unknown argument {s}; expected one of {STYLES:?}, --experimental, --oracle DIR, --corpus DIR, --counts"
                ));
            }
        }
    }
    parsed.oracle = oracle.unwrap_or_else(|| root.join("target/ktlint-oracle").join(parsed.run_name()));
    Ok(parsed)
}

/// Copies the corpus sources to `<staged>/` under one root `.editorconfig`, as ktlint-probe.sh stages them for the
/// oracle, so the corpus repos' own `.editorconfig` files apply to neither side. Returns the staged root and files.
fn stage(corpus: &Path, files: &[PathBuf], staged: &Path, args: &Args) -> Result<(PathBuf, Vec<PathBuf>), String> {
    let _ = fs::remove_dir_all(staged);
    let io = |e: std::io::Error| format!("{}: {e}", staged.display());
    fs::create_dir_all(staged).map_err(io)?;
    let staged = fs::canonicalize(staged).map_err(io)?;
    let mut editorconfig = format!("root = true\n\n[*.{{kt,kts}}]\nktlint_code_style = {}\n", args.style);
    if args.experimental {
        editorconfig.push_str("ktlint_experimental = enabled\n");
    }
    fs::write(staged.join(".editorconfig"), editorconfig).map_err(io)?;
    let files = files
        .iter()
        .map(|file| {
            let target = staged.join(file.strip_prefix(corpus).unwrap());
            fs::create_dir_all(target.parent().unwrap()).map_err(io)?;
            // The oracle ran on a Linux (LF) checkout; a Windows one may have CRLF, which ktlint would keep.
            let text = fs::read(file).map_err(io)?;
            let text = String::from_utf8_lossy(&text).replace("\r\n", "\n");
            fs::write(&target, text).map_err(io)?;
            Ok(target)
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok((staged, files))
}

pub(crate) fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let args = parse_args(root, args)?;
    let providers = standard_rule_providers();
    let ported: HashSet<&str> = providers.iter().map(|p| p.rule_id().value()).collect();
    let oracle = lint_oracle::load(&args.oracle).map_err(|e| {
        format!(
            "no oracle ({e}); build it first (JVM, slow; see research/18-ktlint-corpus-counts.md):\n  \
             KTLINT_CODE_STYLE={} tools/ktlint-oracle/ktlint-probe.sh corpus {}",
            args.style,
            args.oracle.display()
        )
    })?;
    if args.counts {
        lint_oracle::print_counts(&oracle, &ported);
        return Ok(());
    }
    let same_rules = oracle.rules.as_ref().is_some_and(|r| r.iter().map(String::as_str).collect::<HashSet<_>>() == ported);
    let corpus = fs::canonicalize(&args.corpus).map_err(|e| format!("{}: {e}", args.corpus.display()))?;
    let mut sources = Vec::new();
    collect(&corpus, &mut sources);
    sources.sort();
    let (corpus, files) = stage(&corpus, &sources, &root.join("target/lint-diff").join(args.run_name()), &args)?;
    let engine = KtLintRuleEngine::new(providers.clone());

    let next = AtomicUsize::new(0);
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let mut results: Vec<FileResult> = thread::scope(|s| {
        let workers: Vec<_> = (0..thread::available_parallelism().map_or(4, |n| n.get()))
            .map(|_| {
                // Deep trees recurse through the rule traversal (the JVM oracle gives its threads 512 MB).
                thread::Builder::new()
                    .stack_size(256 << 20)
                    .spawn_scoped(s, || {
                        let mut out = Vec::new();
                        while let Some(file) = files.get(next.fetch_add(1, Ordering::Relaxed)) {
                            let rel = file.strip_prefix(&corpus).unwrap().to_string_lossy().replace('\\', "/");
                            out.push(check(&engine, &oracle, &ported, same_rules, file, rel));
                        }
                        out
                    })
                    .unwrap()
            })
            .collect();
        workers.into_iter().flat_map(|w| w.join().unwrap()).collect()
    });
    panic::set_hook(previous_hook);
    results.sort_by(|a, b| a.rel.cmp(&b.rel));
    report(root, &args.run_name(), &results, &oracle, &ported, same_rules, files.len())
}

fn row(e: &LintError) -> Row {
    let auto = if e.can_be_auto_corrected { "auto" } else { "manual" };
    format!("{}\t{}\t{}\t{auto}\t{}", e.line, e.col, e.rule_id.value(), escape(&e.detail))
}

fn check(engine: &KtLintRuleEngine, oracle: &Oracle, ported: &HashSet<&str>, same_rules: bool, file: &Path, rel: String) -> FileResult {
    let mut result = FileResult { rel, ..FileResult::default() };
    let Ok(content) = fs::read_to_string(file) else { return result };
    let code = Code::from_file_content(file, content.clone());
    let run = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut lint = Vec::new();
        let linted = engine.lint(&code, &mut |e| lint.push(row(e)));
        let formatted = (same_rules || oracle.crashed.contains_key(&result.rel)).then(|| {
            let mut rows = Vec::new();
            let text = engine.format(&code, &mut |e| {
                rows.push(row(e));
                AutocorrectDecision::AllowAutocorrect
            });
            text.map(|t| (rows, t))
        });
        (linted.map(|()| lint), formatted)
    }));
    let (lint, formatted) = match run {
        Ok(r) => r,
        Err(payload) => {
            let message = payload.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| payload.downcast_ref::<String>().cloned());
            result.panic = Some(message.unwrap_or_else(|| "<non-string panic>".to_owned()));
            return result;
        }
    };
    let oracle_rejected = oracle.parse_failed.contains(&result.rel);
    let ours = match (lint, oracle_rejected) {
        (Err(_), true) => {
            result.both_rejected = true;
            return result;
        }
        (Err(e), false) => {
            result.rejected_mismatch = Some(format!("we rejected: {e:?}"));
            return result;
        }
        (Ok(_), true) => {
            result.rejected_mismatch = Some("ktlint rejects this file (parse), we linted it".to_owned());
            return result;
        }
        (Ok(rows), false) => rows,
    };
    // Suppression rows depend on the loaded rule set ("unknown or not loaded rule"): comparable only on the same set.
    let compared = |r: &&Row| ported.contains(rule_of(r)) || (same_rules && rule_of(r) == SUPPRESSION_RULE);
    let theirs: Vec<&Row> = oracle.lint.get(&result.rel).map(|rows| rows.iter().filter(compared).collect()).unwrap_or_default();
    (result.missing, result.extra) = multiset_diff(&theirs, &ours.iter().filter(compared).collect::<Vec<_>>());
    if !result.missing.is_empty() || !result.extra.is_empty() {
        let kdoc = kdoc_lines(&content);
        let line_of = |r: &Row| r.split('\t').next().and_then(|l| l.parse::<usize>().ok()).unwrap_or(0);
        result.kdoc_only = result.missing.iter().chain(&result.extra).all(|r| kdoc.contains(&line_of(r)));
    }
    match (oracle.crashed.get(&result.rel), formatted) {
        (Some(failure), Some(formatted)) => result.crash = Some(crash_diff(failure, formatted.err())),
        (None, Some(Ok((rows, text)))) => {
            let expected_rows = oracle.format.get(&result.rel).cloned().unwrap_or_default();
            let expected_text =
                fs::read_to_string(oracle.dir.join("fmt").join(&result.rel)).unwrap_or_else(|_| content.clone());
            result.format_diff = if rows != expected_rows {
                Some(format!("format rows\n  expected: {expected_rows:?}\n  actual:   {rows:?}"))
            } else if text != expected_text {
                Some(first_difference(&expected_text, &text))
            } else {
                None
            };
        }
        (None, Some(Err(e))) => result.format_diff = Some(format!("format threw, ktlint did not: {e}")),
        (_, None) => {}
    }
    result
}

/// The oracle's format threw on this file (failed.tsv `rule\t<id> <exception>`); ours must throw the same. Java prints
/// the exception class qualified, our panics name it unqualified.
fn crash_diff(failure: &str, ours: Option<KtLintException>) -> Option<String> {
    let expected = failure.replacen(" java.lang.", " ", 1);
    let actual = match ours {
        Some(KtLintException::Rule(e)) => format!("rule\t{} {}", e.rule_id, escape(&e.cause)),
        Some(e) => format!("other\t{e}"),
        None => "no exception".to_owned(),
    };
    (actual != expected).then(|| format!("format crash differs\n  expected: {expected}\n  actual:   {actual}"))
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\t', "\\t").replace('\n', "\\n")
}

fn multiset_diff(theirs: &[&Row], ours: &[&Row]) -> (Vec<Row>, Vec<Row>) {
    let mut count: HashMap<&str, isize> = HashMap::new();
    theirs.iter().for_each(|r| *count.entry(r.as_str()).or_default() += 1);
    ours.iter().for_each(|r| *count.entry(r.as_str()).or_default() -= 1);
    let (mut missing, mut extra) = (Vec::new(), Vec::new());
    for (r, n) in count {
        let side = if n > 0 { &mut missing } else { &mut extra };
        (0..n.unsigned_abs()).for_each(|_| side.push(r.to_owned()));
    }
    missing.sort();
    extra.sort();
    (missing, extra)
}

/// 1-based lines touched by a `/** ... */` comment (a text scan; good enough to label suspects).
fn kdoc_lines(text: &str) -> HashSet<usize> {
    let mut lines = HashSet::new();
    let mut rest = text;
    let mut offset = 0;
    while let Some(start) = rest.find("/**") {
        let end = rest[start..].find("*/").map_or(rest.len(), |e| start + e + 2);
        let first = text[..offset + start].matches('\n').count() + 1;
        let last = first + rest[start..end].matches('\n').count();
        lines.extend(first..=last);
        offset += end;
        rest = &rest[end..];
    }
    lines
}

fn first_difference(expected: &str, actual: &str) -> String {
    let (mut e, mut a) = (expected.split('\n'), actual.split('\n'));
    for line in 1.. {
        match (e.next(), a.next()) {
            (x, y) if x == y && x.is_some() => {}
            (x, y) => {
                let show = |s: Option<&str>| s.map_or("<eof>".to_owned(), |s| format!("{s:?}"));
                return format!("formatted text, line {line}\n  expected: {}\n  actual:   {}", show(x), show(y));
            }
        }
    }
    unreachable!()
}
