//! `compare <jvm-out> <rust-out>`: the go/no-go diff. Per file where either side reports a violation (or
//! fails), compares format rows, lint rows, per-pass changed flags, failure kind, formatted text and every
//! mutated-tree dump (header line skipped: it carries each side's own path).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

type Table = BTreeMap<String, Vec<String>>;

/// Rows of a `file\t...` table grouped by file, the file column stripped; `keep` picks the columns kept.
fn table(path: &Path, header: bool, keep: impl Fn(&[&str]) -> String) -> Table {
    let mut rows = Table::new();
    let text = fs::read_to_string(path).unwrap_or_default();
    for line in text.lines().skip(usize::from(header)) {
        let cols: Vec<&str> = line.split('\t').collect();
        rows.entry(cols[0].to_owned()).or_default().push(keep(&cols[1..]));
    }
    rows
}

fn tables(out: &Path) -> [Table; 4] {
    [
        table(&out.join("format.tsv"), true, |c| c.join("\t")),
        table(&out.join("lint.tsv"), true, |c| c.join("\t")),
        table(&out.join("passes.tsv"), true, |c| format!("{}\t{}", c[0], c[1])),
        table(&out.join("failed.tsv"), false, |c| c[0].to_owned()),
    ]
}

fn read(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|b| String::from_utf8_lossy(&b).into_owned())
}

fn without_header(dump: Option<String>) -> Option<String> {
    dump.map(|d| d.split_once('\n').map_or(String::new(), |(_, rest)| rest.to_owned()))
}

/// `src` holds the sources by relative path (the oracle's staged `<jvm>/src` when the out-dir kept it).
pub fn compare(jvm: &Path, rust: &Path, src: &Path) -> i32 {
    let [jvm_format, jvm_lint, jvm_passes, jvm_failed] = tables(jvm);
    let [rust_format, rust_lint, rust_passes, rust_failed] = tables(rust);
    let files: BTreeSet<&String> =
        jvm_format.keys().chain(rust_format.keys()).chain(jvm_failed.keys()).chain(rust_failed.keys()).collect();
    let mut mismatches = String::from("file\twhat\tsuspect\n");
    let mut bad = 0;
    for &file in &files {
        let mut what = Vec::new();
        for (name, a, b) in [
            ("format", &jvm_format, &rust_format),
            ("lint", &jvm_lint, &rust_lint),
            ("passes", &jvm_passes, &rust_passes),
            ("failed", &jvm_failed, &rust_failed),
        ] {
            if a.get(file) != b.get(file) {
                what.push(name.to_owned());
            }
        }
        if read(&jvm.join("fmt").join(file)) != read(&rust.join("fmt").join(file)) {
            what.push("fmt".to_owned());
        }
        for pass in 1..=4 {
            let dump = format!("mut/{file}.p{pass}.txt");
            if without_header(read(&jvm.join(&dump))) != without_header(read(&rust.join(&dump))) {
                what.push(format!("mut.p{pass}"));
            }
        }
        if !what.is_empty() {
            bad += 1;
            mismatches.push_str(&format!("{file}\t{}\t{}\n", what.join(","), suspect(&src.join(file))));
        }
    }
    fs::write(rust.join("mismatches.tsv"), &mismatches).unwrap();
    println!("{} files with a violation or failure on either side; {} identical, {bad} differ", files.len(), files.len() - bad);
    for line in mismatches.lines().skip(1).take(20) {
        println!("  {line}");
    }
    i32::from(bad != 0)
}

/// A mismatch in a file the port knowingly handles differently (see research/15-ktlint-spike.md, "Missing").
fn suspect(source: &Path) -> &'static str {
    match read(source) {
        Some(text) if text.contains("ktlint") => "ktlint-directive (suppression TODO)",
        Some(text) if text.contains("@formatter:") => "formatter-tag (suppression TODO)",
        Some(_) => "",
        None => "no staged source",
    }
}

/// `hunk(a, b, max)` of tools/ktlint-oracle/src/Probe.kt: one hunk from the first to the last differing line.
pub fn hunk(a: &str, b: &str, max: usize) -> String {
    let x: Vec<&str> = a.split('\n').collect();
    let y: Vec<&str> = b.split('\n').collect();
    let mut p = 0;
    while p < x.len() && p < y.len() && x[p] == y[p] {
        p += 1;
    }
    let mut s = 0;
    while s < x.len() - p && s < y.len() - p && x[x.len() - 1 - s] == y[y.len() - 1 - s] {
        s += 1;
    }
    let mut out = format!("@@ -{},{} +{},{} @@ (- mutated, + reparse)\n", p + 1, x.len() - p - s, p + 1, y.len() - p - s);
    for line in &x[p.saturating_sub(4)..p] {
        out.push_str(&format!(" {line}\n"));
    }
    for line in x[p..x.len() - s].iter().take(max) {
        out.push_str(&format!("-{line}\n"));
    }
    for line in y[p..y.len() - s].iter().take(max) {
        out.push_str(&format!("+{line}\n"));
    }
    out
}
