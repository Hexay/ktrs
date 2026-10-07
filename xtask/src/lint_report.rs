//! `lint-diff`'s per-file results and its summary: stdout counts plus target/lint-diff-<style>.txt.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::lint_oracle::{Oracle, Row, SUPPRESSION_RULE, rule_of};

#[derive(Default)]
pub(crate) struct FileResult {
    pub rel: String,
    pub missing: Vec<Row>,
    pub extra: Vec<Row>,
    /// Every mismatched row is on a KDoc line: likely the oracle jar's Kotlin 2.4.10 KDoc lexer, not a rule.
    pub kdoc_only: bool,
    pub format_diff: Option<String>,
    pub rejected_mismatch: Option<String>,
    pub both_rejected: bool,
    pub panic: Option<String>,
    /// The oracle's format threw on this file: `Some(None)` when ours throws the same, `Some(Some(diff))` otherwise.
    pub crash: Option<Option<String>>,
}

pub(crate) fn report(
    root: &Path,
    style: &str,
    results: &[FileResult],
    oracle: &Oracle,
    ported: &HashSet<&str>,
    same_rules: bool,
    total: usize,
) -> Result<(), String> {
    let mut per_rule: BTreeMap<&str, [usize; 2]> = ported.iter().map(|r| (*r, [0, 0])).collect();
    if same_rules {
        per_rule.insert(SUPPRESSION_RULE, [0, 0]);
    }
    let (mut lint_same, mut lint_differ, mut kdoc, mut fmt_same, mut fmt_differ) = (0, 0, 0, 0, 0);
    let (mut panics, mut mismatched, mut both_rejected, mut crash_same) = (0, 0, 0, 0);
    let mut text = String::new();
    for r in results {
        match &r.crash {
            Some(None) => crash_same += 1,
            Some(Some(diff)) => text.push_str(&format!("{}\n  {diff}\n\n", r.rel)),
            None => {}
        }
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
        if same_rules && r.crash.is_none() {
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
         rejected-mismatch {mismatched}  both-rejected {both_rejected}/{}  oracle-crash {} (format throws the same: {crash_same})",
        oracle.parse_failed.len(),
        oracle.crashed.len(),
    );
    if same_rules {
        println!("format: identical {fmt_same}  differ {fmt_differ}");
    } else {
        let mut rules: Vec<&str> = ported.iter().copied().collect();
        rules.sort();
        println!(
            "format and {SUPPRESSION_RULE}: not compared (the oracle's rule set is not the ported one); for them build\n  \
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
        return Err(format!("differences in {}", report_path.display()));
    }
    Ok(())
}
