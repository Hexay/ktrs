//! The engine against the detekt jar on the smoke files of tools/detekt-oracle/smoke/cases.txt: location semantics
//! (UTF-16 columns, CRLF, lone CR, BOM, empty files, no final newline), suppression ids and aliases, path filters,
//! scripts, syntax errors and MaxLineLength's exemptions. `tests/data/smoke.jvm.tsv` holds the jar's rows of the
//! default config; regenerate it with the command in tools/detekt-oracle/smoke.sh. Rows of unported rules are left
//! out; rows are compared per file as multisets (detekt's order across rule sets follows its service loader).

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use ktrs_detekt::config::{load_configuration, workaround_configuration};
use ktrs_detekt::engine::create_analyzer;
use ktrs_detekt::probe::{kotlin_files, rows_of_file};
use ktrs_detekt::rules::default_rule_set_providers;

/// The `\n \r \t \\ \xHH` escapes and `{xN}` runs of cases.txt (tools/detekt-oracle/smoke.sh decodes the same).
fn decode(content: &str) -> Vec<u8> {
    let bytes = content.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            let (byte, len) = match bytes[i + 1] {
                b'n' => (b'\n', 2),
                b'r' => (b'\r', 2),
                b't' => (b'\t', 2),
                b'\\' => (b'\\', 2),
                b'x' => (u8::from_str_radix(std::str::from_utf8(&bytes[i + 2..i + 4]).unwrap(), 16).unwrap(), 4),
                _ => (b'\\', 1),
            };
            out.push(byte);
            i += len;
        } else if bytes[i..].starts_with(b"{x")
            && let Some(end) = bytes[i..].iter().position(|&b| b == b'}')
            && let Some(count) = std::str::from_utf8(&bytes[i + 2..i + end]).ok().and_then(|digits| digits.parse::<usize>().ok())
        {
            out.extend(std::iter::repeat_n(b'x', count));
            i += end + 1;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    out
}

fn write_cases(dir: &Path) {
    let cases = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/detekt-oracle/smoke/cases.txt");
    for line in fs::read_to_string(cases).unwrap().lines() {
        let (path, content) = line.split_once('|').unwrap();
        let target = dir.join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, decode(content)).unwrap();
    }
}

#[test]
fn smoke_rows_match_the_jar() {
    let dir: PathBuf = std::env::temp_dir().join(format!("ktrs-detekt-smoke-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    write_cases(&dir);
    let base_path = fs::canonicalize(&dir).unwrap();

    let ported: HashSet<String> =
        default_rule_set_providers().iter().flat_map(|p| (p.instance)().rules).map(|(name, _)| name.value().to_owned()).collect();
    let expected_text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/smoke.jvm.tsv")).unwrap();
    let mut expected: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in expected_text.lines() {
        let (file, row) = line.split_once('\t').unwrap();
        if ported.contains(row.split('\t').nth(6).unwrap()) {
            expected.entry(file.to_owned()).or_default().push(row.to_owned());
        }
    }

    let config = workaround_configuration(load_configuration(&[]).unwrap(), false, false, false);
    let analyzer = create_analyzer(base_path.clone(), &config);
    let mut actual: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in kotlin_files(&base_path) {
        let rel = file.strip_prefix(&base_path).unwrap().to_string_lossy().replace('\\', "/");
        let rows = rows_of_file(&analyzer, &file).unwrap_or_else(|message| panic!("{rel}: {message}"));
        if !rows.is_empty() {
            actual.insert(rel, rows);
        }
    }
    let _ = fs::remove_dir_all(&dir);

    expected.values_mut().for_each(|rows| rows.sort());
    actual.values_mut().for_each(|rows| rows.sort());
    let mut differences = String::new();
    let files: HashSet<&String> = expected.keys().chain(actual.keys()).collect();
    let mut files: Vec<&String> = files.into_iter().collect();
    files.sort();
    for file in files {
        let (theirs, ours) = (expected.get(file).cloned().unwrap_or_default(), actual.get(file).cloned().unwrap_or_default());
        if theirs != ours {
            differences.push_str(&format!("{file}\n--- jar\n{}\n--- ktrs\n{}\n", theirs.join("\n"), ours.join("\n")));
        }
    }
    assert!(differences.is_empty(), "smoke rows differ from the jar:\n{differences}");
}
