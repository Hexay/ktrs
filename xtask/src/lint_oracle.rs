//! Reading a KtlintProbe out-dir (tools/ktlint-oracle; layout in research/15-ktlint-spike.md) for `lint-diff`,
//! and the per-rule corpus counts (`cargo lint-diff <style> --counts`).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// The engine's internal rule: always loaded by the oracle, whatever `--rules` says.
pub(crate) const SUPPRESSION_RULE: &str = "internal:ktlint-suppression";

/// A `line\tcol\trule\tauto\tdetail` row, as both sides print it.
pub(crate) type Row = String;

pub(crate) struct Oracle {
    pub dir: PathBuf,
    /// `rules=` of the summary header; `None` = the whole standard rule set.
    pub rules: Option<Vec<String>>,
    pub lint: HashMap<String, Vec<Row>>,
    /// Format-callback rows without the pass column, in emit order.
    pub format: HashMap<String, Vec<Row>>,
    pub parse_failed: HashSet<String>,
    /// Rule or engine crashes (`rule`/`crash` rows of failed.tsv), keyed by file: `<kind>\t<escaped exception>`.
    /// The oracle has no lint or format result for these files; the port must throw the same.
    pub crashed: HashMap<String, String>,
}

pub(crate) fn rule_of(row: &str) -> &str {
    row.split('\t').nth(2).unwrap_or("")
}

fn by_file(path: &Path, skip: usize) -> Result<HashMap<String, Vec<Row>>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut rows: HashMap<String, Vec<Row>> = HashMap::new();
    for line in text.lines().skip(1) {
        let (file, rest) = line.split_once('\t').ok_or_else(|| format!("{}: bad row {line}", path.display()))?;
        let rest = rest.splitn(skip + 1, '\t').last().unwrap();
        rows.entry(file.to_owned()).or_default().push(rest.to_owned());
    }
    Ok(rows)
}

pub(crate) fn load(dir: &Path) -> Result<Oracle, String> {
    let summary = fs::read_to_string(dir.join("summary.txt")).map_err(|e| format!("{}: {e}", dir.display()))?;
    let rules = summary
        .lines()
        .next()
        .and_then(|l| l.split_once("rules=").map(|(_, r)| r.trim()))
        .filter(|r| !r.starts_with("standard (all)"))
        .map(|r| r.split(',').map(str::to_owned).collect());
    let failed = fs::read_to_string(dir.join("failed.tsv")).unwrap_or_default();
    let (mut parse_failed, mut crashed) = (HashSet::new(), HashMap::new());
    for line in failed.lines() {
        let Some((file, failure)) = line.split_once('\t') else { continue };
        match failure.split('\t').next() {
            Some("parse") => {
                parse_failed.insert(file.to_owned());
            }
            Some("rule" | "crash") => {
                crashed.insert(file.to_owned(), failure.to_owned());
            }
            _ => {}
        }
    }
    Ok(Oracle { dir: dir.to_owned(), rules, lint: by_file(&dir.join("lint.tsv"), 0)?, format: by_file(&dir.join("format.tsv"), 1)?, parse_failed, crashed })
}

/// Per-rule lint violations on the corpus: rows, files, autocorrectable rows. Sorted by rows, descending.
pub(crate) fn print_counts(oracle: &Oracle, ported: &HashSet<&str>) {
    let mut counts: BTreeMap<&str, (usize, HashSet<&str>, usize)> = BTreeMap::new();
    for (file, rows) in &oracle.lint {
        for row in rows {
            let entry = counts.entry(rule_of(row)).or_default();
            entry.0 += 1;
            entry.1.insert(file);
            entry.2 += usize::from(row.split('\t').nth(3) == Some("auto"));
        }
    }
    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.0.cmp(&a.1.0).then(a.0.cmp(b.0)));
    let total: usize = sorted.iter().map(|(_, c)| c.0).sum();
    println!("rank\trule\tviolations\tfiles\tautocorrectable\tported");
    for (i, (rule, (rows, files, auto))) in sorted.iter().enumerate() {
        println!("{}\t{rule}\t{rows}\t{}\t{auto}\t{}", i + 1, files.len(), if ported.contains(rule) { "y" } else { "" });
    }
    println!("total\t{} rules\t{total}\t{} files with a violation", sorted.len(), oracle.lint.len());
}
