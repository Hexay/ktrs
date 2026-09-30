//! `cargo lint-diff [ktlint_official|intellij_idea|android_studio] [--oracle DIR] [--counts] [--corpus DIR]`:
//! lints (and, when the oracle ran exactly the ported rules, formats) every corpus file with ktrs-lint and diffs
//! against the real ktlint engine, pre-built on the JVM by (background, testbox):
//!   KTLINT_CODE_STYLE=<style> tools/ktlint-oracle/ktlint-probe.sh corpus target/ktlint-oracle/<style> [--rules <ported>]
//! Lint rows are compared per file as a multiset over the ported rules plus the engine's suppression rule (lint is
//! per-rule independent, so an all-rules oracle filters down); format rows (emit order) and formatted bytes need an
//! oracle restricted to the ported rules. Mismatches go to target/lint-diff-<style>.txt; `--counts` prints the
//! oracle's per-rule violation counts instead.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fs, thread};

use ktrs_lint::rules::STANDARD_RULE_PROVIDERS;
use ktrs_lint::{AutocorrectDecision, Code, KtLintRuleEngine, LintError};

use crate::corpus_diff::collect;
use crate::lint_oracle::{self, Oracle, Row, SUPPRESSION_RULE, rule_of};

const STYLES: [&str; 3] = ["ktlint_official", "intellij_idea", "android_studio"];

struct Args {
    style: String,
    oracle: PathBuf,
    corpus: PathBuf,
    counts: bool,
}

#[derive(Default)]
struct FileResult {
    rel: String,
    missing: Vec<Row>,
    extra: Vec<Row>,
    /// Every mismatched row is on a KDoc line: likely the oracle jar's Kotlin 2.4.10 KDoc lexer, not a rule.
    kdoc_only: bool,
    format_diff: Option<String>,
    rejected_mismatch: Option<String>,
    both_rejected: bool,
    panic: Option<String>,
}

fn parse_args(root: &Path, args: &[String]) -> Result<Args, String> {
    let mut parsed = Args { style: STYLES[0].to_owned(), oracle: PathBuf::new(), corpus: root.join("corpus"), counts: false };
    let mut oracle = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--oracle" => oracle = rest.next().map(PathBuf::from),
            "--corpus" => parsed.corpus = rest.next().map(PathBuf::from).ok_or("--corpus needs a dir")?,
            "--counts" => parsed.counts = true,
            s if STYLES.contains(&s) => parsed.style = s.to_owned(),
            s => return Err(format!("unknown argument {s}; expected one of {STYLES:?}, --oracle DIR, --corpus DIR, --counts")),
        }
    }
    parsed.oracle = oracle.unwrap_or_else(|| root.join("target/ktlint-oracle").join(&parsed.style));
    Ok(parsed)
}

// TODO: set the code style through ktrs-lint's editorconfig API once it has one; ktlint_official is its hard-wired default.
fn engine_for(style: &str) -> Result<KtLintRuleEngine, String> {
    if style != "ktlint_official" {
        return Err(format!("ktrs-lint can't run code style {style} yet"));
    }
    Ok(KtLintRuleEngine { rule_providers: STANDARD_RULE_PROVIDERS.iter().map(|(_, p)| *p).collect() })
}

pub(crate) fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let args = parse_args(root, args)?;
    let ported: HashSet<&str> = STANDARD_RULE_PROVIDERS.iter().map(|(id, _)| *id).collect();
    let oracle = lint_oracle::load(&args.oracle).map_err(|e| {
        format!(
            "no oracle ({e}); build it first (JVM, slow; see research/17-ktlint-corpus-counts.md):\n  \
             KTLINT_CODE_STYLE={} tools/ktlint-oracle/ktlint-probe.sh corpus {}",
            args.style,
            args.oracle.display()
        )
    })?;
    if args.counts {
        lint_oracle::print_counts(&oracle, &ported);
        return Ok(());
    }
    let engine = engine_for(&args.style)?;
    let compare_format = oracle.rules.as_ref().is_some_and(|r| r.iter().map(String::as_str).collect::<HashSet<_>>() == ported);
    let corpus = fs::canonicalize(&args.corpus).map_err(|e| format!("{}: {e}", args.corpus.display()))?;
    let mut files = Vec::new();
    collect(&corpus, &mut files);
    files.sort();

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
                            if !oracle.crashed.contains(&rel) {
                                out.push(check(&engine, &oracle, &ported, compare_format, file, rel));
                            }
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
    report(root, &args.style, &results, &oracle, &ported, compare_format, files.len())
}

fn report(
    root: &Path,
    style: &str,
    results: &[FileResult],
    oracle: &Oracle,
    ported: &HashSet<&str>,
    compare_format: bool,
    total: usize,
) -> Result<(), String> {
    let mut per_rule: BTreeMap<&str, [usize; 2]> = ported.iter().map(|r| (*r, [0, 0])).collect();
    per_rule.insert(SUPPRESSION_RULE, [0, 0]);
    let (mut lint_same, mut lint_differ, mut kdoc, mut fmt_same, mut fmt_differ) = (0, 0, 0, 0, 0);
    let (mut panics, mut mismatched, mut both_rejected) = (0, 0, 0);
    let mut text = String::new();
    for r in results {
        if let Some(message) = &r.panic {
            panics += 1;
            text.push_str(&format!("{}\n  panic: {message}\n\n", r.rel));
            continue;
        }
        if let Some(what) = &r.rejected_mismatch {
            mismatched += 1;
            text.push_str(&format!("{}\n  {what}\n\n", r.rel));
            continue;
        }
        if r.both_rejected {
            both_rejected += 1;
            continue;
        }
        let lint_ok = r.missing.is_empty() && r.extra.is_empty();
        match (lint_ok, r.kdoc_only) {
            (true, _) => lint_same += 1,
            (false, true) => kdoc += 1,
            (false, false) => lint_differ += 1,
        }
        for (i, rows) in [&r.missing, &r.extra].into_iter().enumerate() {
            rows.iter().for_each(|row| per_rule.entry(rule_of(row)).or_default()[i] += 1);
        }
        if compare_format {
            if r.format_diff.is_some() { fmt_differ += 1 } else { fmt_same += 1 }
        }
        if lint_ok && r.format_diff.is_none() {
            continue;
        }
        text.push_str(&format!("{}{}\n", r.rel, if r.kdoc_only { "  (kdoc lines only: 2.4.10 KDoc lexer?)" } else { "" }));
        r.missing.iter().for_each(|row| text.push_str(&format!("  - {row}\n")));
        r.extra.iter().for_each(|row| text.push_str(&format!("  + {row}\n")));
        if let Some(diff) = &r.format_diff {
            text.push_str(&format!("  {diff}\n"));
        }
        text.push('\n');
    }
    let report_path = root.join(format!("target/lint-diff-{style}.txt"));
    fs::write(&report_path, &text).map_err(|e| e.to_string())?;

    let oracle_rows: HashMap<&str, usize> = oracle.lint.values().flatten().fold(HashMap::new(), |mut m, row| {
        *m.entry(rule_of(row)).or_default() += 1;
        m
    });
    println!(
        "{style}: lint identical {lint_same}/{total}  differ {lint_differ}  kdoc-pin-suspect {kdoc}  panic {panics}  \
         rejected-mismatch {mismatched}  both-rejected {both_rejected}/{}  oracle-crash {}",
        oracle.parse_failed.len(),
        oracle.crashed.len(),
    );
    if compare_format {
        println!("format: identical {fmt_same}  differ {fmt_differ}");
    } else {
        let mut rules: Vec<&str> = ported.iter().copied().collect();
        rules.sort();
        println!(
            "format: not compared (the oracle's rule set is not the ported one); for it build\n  \
             KTLINT_CODE_STYLE={style} tools/ktlint-oracle/ktlint-probe.sh corpus target/ktlint-oracle/{style}-ported --rules {}\n  \
             and pass --oracle target/ktlint-oracle/{style}-ported",
            rules.join(",")
        );
    }
    println!("rule\toracle rows\tmissing\textra");
    for (rule, [missing, extra]) in &per_rule {
        println!("{rule}\t{}\t{missing}\t{extra}", oracle_rows.get(rule).unwrap_or(&0));
    }
    if !text.is_empty() {
        println!("differences: {}", report_path.display());
    }
    Ok(())
}

fn row(e: &LintError) -> Row {
    let auto = if e.can_be_auto_corrected { "auto" } else { "manual" };
    let detail = e.detail.replace('\\', "\\\\").replace('\t', "\\t").replace('\n', "\\n");
    format!("{}\t{}\t{}\t{auto}\t{detail}", e.line, e.col, e.rule_id.value())
}

fn check(engine: &KtLintRuleEngine, oracle: &Oracle, ported: &HashSet<&str>, compare_format: bool, file: &Path, rel: String) -> FileResult {
    let mut result = FileResult { rel, ..FileResult::default() };
    // The oracle ran on a Linux (LF) checkout; a Windows one may have CRLF, which ktlint would keep.
    let Ok(content) = fs::read_to_string(file).map(|t| t.replace("\r\n", "\n")) else { return result };
    let code = Code::from_file(&file.to_string_lossy(), content.clone());
    let run = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut lint = Vec::new();
        let linted = engine.lint(&code, &mut |e| lint.push(row(e)));
        let formatted = compare_format.then(|| {
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
    let compared = |r: &&Row| ported.contains(rule_of(r)) || rule_of(r) == SUPPRESSION_RULE;
    let theirs: Vec<&Row> = oracle.lint.get(&result.rel).map(|rows| rows.iter().filter(compared).collect()).unwrap_or_default();
    (result.missing, result.extra) = multiset_diff(&theirs, &ours.iter().collect::<Vec<_>>());
    if !result.missing.is_empty() || !result.extra.is_empty() {
        let kdoc = kdoc_lines(&content);
        let line_of = |r: &Row| r.split('\t').next().and_then(|l| l.parse::<usize>().ok()).unwrap_or(0);
        result.kdoc_only = result.missing.iter().chain(&result.extra).all(|r| kdoc.contains(&line_of(r)));
    }
    if let Some(Ok((rows, text))) = formatted {
        let expected_rows = oracle.format.get(&result.rel).cloned().unwrap_or_default();
        let expected_text = fs::read_to_string(oracle.dir.join("fmt").join(&result.rel)).unwrap_or_else(|_| content.clone());
        result.format_diff = if rows != expected_rows {
            Some(format!("format rows\n  expected: {expected_rows:?}\n  actual:   {rows:?}"))
        } else if text != expected_text {
            Some(first_difference(&expected_text, &text))
        } else {
            None
        };
    }
    result
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
