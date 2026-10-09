//! `cargo detekt-diff [default|all-rules] [--oracle DIR] [--corpus DIR] [--counts]`: runs ktrs-detekt (light mode,
//! default config) on every corpus file and diffs its rows against the detekt jar's, pre-built on the JVM by
//! (background, testbox):
//!   tools/detekt-oracle/detekt-probe.sh corpus target/detekt-oracle/default
//!   tools/detekt-oracle/detekt-probe.sh corpus target/detekt-oracle/all-rules --all-rules
//! Rows (line, column, end, offsets, rule, severity, signature, message) are compared per file as multisets over
//! the ported rules: detekt's order inside a file is not defined for every rule (research/33). Accepted
//! mismatches are the rows of tools/parity/known-diffs/detekt.tsv (`<run>\t<-|+>\t<file>\t<row>`; a listed row that
//! no longer differs fails as stale). Mismatches go to target/detekt-diff-<run>.txt and exit 1; `--counts` prints the
//! oracle's per-rule totals instead.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fs, panic, thread};

use ktrs_detekt::config::{load_configuration, workaround_configuration};
use ktrs_detekt::engine::create_analyzer;
use ktrs_detekt::probe::{kotlin_files, rows_of_file};
use ktrs_detekt::rules::default_rule_set_providers;

const RUNS: [&str; 2] = ["default", "all-rules"];

/// The rule id of a row without its file column.
fn rule_of(row: &str) -> &str {
    row.split('\t').nth(6).unwrap_or("")
}

/// `<file> -> rows` of a `rows.tsv`.
fn load_oracle(dir: &Path) -> Result<HashMap<String, Vec<String>>, String> {
    let path = dir.join("rows.tsv");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut rows: HashMap<String, Vec<String>> = HashMap::new();
    for line in text.lines() {
        let (file, row) = line.split_once('\t').ok_or_else(|| format!("{}: malformed row {line}", path.display()))?;
        rows.entry(file.to_owned()).or_default().push(row.to_owned());
    }
    Ok(rows)
}

/// The accepted `(sign, file, row)` of `run`.
fn load_known(root: &Path, run: &str) -> HashSet<(String, String, String)> {
    let text = fs::read_to_string(root.join("tools/parity/known-diffs/detekt.tsv")).unwrap_or_default();
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(4, '\t');
            let (line_run, sign, file, row) = (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
            (line_run == run).then(|| (sign.to_owned(), file.to_owned(), row.to_owned()))
        })
        .collect()
}

/// `(missing, extra)`: rows only the oracle has, rows only we have.
fn multiset_diff(theirs: &[&String], ours: &[&String]) -> (Vec<String>, Vec<String>) {
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

pub(crate) fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let (mut run, mut oracle_dir, mut corpus, mut counts) = (RUNS[0].to_owned(), None, root.join("corpus"), false);
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--oracle" => oracle_dir = rest.next().map(PathBuf::from),
            "--corpus" => corpus = rest.next().map(PathBuf::from).ok_or("--corpus needs a dir")?,
            "--counts" => counts = true,
            s if RUNS.contains(&s) => run = s.to_owned(),
            s => return Err(format!("unknown argument {s}; expected one of {RUNS:?}, --oracle DIR, --corpus DIR, --counts")),
        }
    }
    let oracle_dir = oracle_dir.unwrap_or_else(|| root.join("target/detekt-oracle").join(&run));
    let all_rules = run == "all-rules";
    let oracle = load_oracle(&oracle_dir).map_err(|e| {
        format!(
            "no oracle ({e}); build it first (JVM, background):\n  tools/detekt-oracle/detekt-probe.sh corpus {}{}",
            oracle_dir.display(),
            if all_rules { " --all-rules" } else { "" }
        )
    })?;
    let ported: HashSet<String> =
        default_rule_set_providers().iter().flat_map(|p| (p.instance)().rules).map(|(name, _)| name.value().to_owned()).collect();
    let mut oracle_rows: BTreeMap<&str, usize> = BTreeMap::new();
    oracle.values().flatten().for_each(|row| *oracle_rows.entry(rule_of(row)).or_default() += 1);
    if counts {
        println!("rule\toracle rows\tported");
        let mut by_count: Vec<_> = oracle_rows.iter().collect();
        by_count.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        by_count.iter().for_each(|(rule, n)| println!("{rule}\t{n}\t{}", if ported.contains(**rule) { "yes" } else { "no" }));
        return Ok(());
    }

    let corpus = fs::canonicalize(&corpus).map_err(|e| format!("{}: {e}", corpus.display()))?;
    let config = workaround_configuration(load_configuration(&[])?, all_rules, false, false);
    let analyzer = create_analyzer(corpus.clone(), &config);
    let files = kotlin_files(&corpus);
    let next = AtomicUsize::new(0);
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let mut results: Vec<(String, Result<Vec<String>, String>)> = thread::scope(|s| {
        let workers: Vec<_> = (0..thread::available_parallelism().map_or(4, |n| n.get()))
            .map(|_| {
                thread::Builder::new()
                    .stack_size(256 << 20)
                    .spawn_scoped(s, || {
                        let mut out = Vec::new();
                        while let Some(file) = files.get(next.fetch_add(1, Ordering::Relaxed)) {
                            let rel = file.strip_prefix(&corpus).unwrap().to_string_lossy().replace('\\', "/");
                            out.push((rel, rows_of_file(&analyzer, file)));
                        }
                        out
                    })
                    .unwrap()
            })
            .collect();
        workers.into_iter().flat_map(|w| w.join().unwrap()).collect()
    });
    panic::set_hook(previous_hook);
    results.sort_by(|a, b| a.0.cmp(&b.0));

    let known = load_known(root, &run);
    let mut seen_known = HashSet::new();
    let mut per_rule: BTreeMap<String, [usize; 2]> = ported.iter().map(|r| (r.clone(), [0, 0])).collect();
    let (mut identical, mut differ, mut known_only, mut panics, mut ours_total) = (0, 0, 0, 0, 0);
    let mut text = String::new();
    for (rel, rows) in &results {
        let rows = match rows {
            Ok(rows) => rows,
            Err(message) => {
                panics += 1;
                text.push_str(&format!("{rel}\n  panic: {message}\n\n"));
                continue;
            }
        };
        ours_total += rows.len();
        let theirs: Vec<&String> = oracle.get(rel).map(|rows| rows.iter().filter(|r| ported.contains(rule_of(r))).collect()).unwrap_or_default();
        let (missing, extra) = multiset_diff(&theirs, &rows.iter().collect::<Vec<_>>());
        let mut unknown = Vec::new();
        for (sign, row) in missing.iter().map(|r| ("-", r)).chain(extra.iter().map(|r| ("+", r))) {
            let key = (sign.to_owned(), rel.clone(), row.clone());
            if known.contains(&key) {
                seen_known.insert(key);
            } else {
                per_rule.entry(rule_of(row).to_owned()).or_default()[usize::from(sign == "+")] += 1;
                unknown.push(format!("  {sign} {row}\n"));
            }
        }
        match (missing.is_empty() && extra.is_empty(), unknown.is_empty()) {
            (true, _) => identical += 1,
            (false, true) => known_only += 1,
            (false, false) => {
                differ += 1;
                text.push_str(&format!("{rel}\n{}\n", unknown.concat()));
            }
        }
    }
    let stale: Vec<_> = known.difference(&seen_known).collect();
    for (sign, file, row) in &stale {
        text.push_str(&format!("stale known diff: {sign} {file} {row}\n"));
    }
    let report_path = root.join(format!("target/detekt-diff-{run}.txt"));
    fs::write(&report_path, &text).map_err(|e| e.to_string())?;

    let oracle_ported: usize = oracle_rows.iter().filter(|(rule, _)| ported.contains(**rule)).map(|(_, n)| n).sum();
    println!(
        "{run}: rows identical in {identical}/{} files  differ {differ}  known-diffs-only {known_only}  panic {panics}  \
         stale known diffs {}  (oracle rows of ported rules {oracle_ported}, ours {ours_total})",
        results.len(),
        stale.len()
    );
    println!("rule\toracle rows\tmissing\textra");
    for (rule, [missing, extra]) in &per_rule {
        println!("{rule}\t{}\t{missing}\t{extra}", oracle_rows.get(rule.as_str()).unwrap_or(&0));
    }
    if !text.is_empty() {
        println!("differences: {}", report_path.display());
        return Err(format!("differences in {}", report_path.display()));
    }
    Ok(())
}
