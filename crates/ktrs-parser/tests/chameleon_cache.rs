//! `parse_file_cached` must equal `parse_file` whatever the cache holds: one cache is shared by every
//! fixture (so hits cross files and contexts), and each fixture is parsed cold and then warm.

use std::path::{Path, PathBuf};
use std::{env, fs};

use ktrs_parser::{ChameleonCache, FileKind, parse_file, parse_file_cached};

#[test]
fn cached_parse_equals_uncached() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/kotlin/psi");
    let mut files = Vec::new();
    collect(&dir, &mut files);
    files.sort();
    assert!(!files.is_empty(), "no fixtures under {}; run tools/sync-kotlin.sh", dir.display());

    let mut cache = ChameleonCache::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap().replace("\r\n", "\n");
        let kind = FileKind::from_file_name(&file.to_string_lossy());
        let expected = parse_file(&text, kind);
        for pass in ["cold", "warm"] {
            let actual = parse_file_cached(&text, kind, &mut cache);
            assert_eq!(actual.tree, expected.tree, "{pass} tree differs: {}", file.display());
            assert_eq!(actual.error_messages, expected.error_messages, "{pass} errors differ: {}", file.display());
        }
    }
}

/// After a fixture fills the cache, whitespace-perturbed variants (the formatter's typical edit)
/// parse through it: bodies whose text survived are hits, the rest reparse; both must equal an
/// uncached parse.
#[test]
fn whitespace_edited_parse_equals_uncached() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/kotlin/psi");
    let mut files = Vec::new();
    collect(&dir, &mut files);
    files.sort();
    let variants: [(&str, fn(&str) -> String); 5] = [
        ("double spaces", |t| t.replace(' ', "  ")),
        ("blank lines", |t| t.replace('\n', "\n\n")),
        ("joined lines", |t| t.replace('\n', " ")),
        ("breaks after ( and ,", |t| t.replace('(', "(\n").replace(',', ",\n")),
        ("indent", |t| t.replace('\n', "\n    ")),
    ];
    for file in &files {
        let text = fs::read_to_string(file).unwrap().replace("\r\n", "\n");
        let kind = FileKind::from_file_name(&file.to_string_lossy());
        for (name, perturb) in &variants {
            let mut cache = ChameleonCache::new();
            parse_file_cached(&text, kind, &mut cache);
            let variant = perturb(&text);
            let expected = parse_file(&variant, kind);
            let actual = parse_file_cached(&variant, kind, &mut cache);
            assert_eq!(actual.tree, expected.tree, "{name}: tree differs: {}", file.display());
            assert_eq!(actual.error_messages, expected.error_messages, "{name}: errors differ: {}", file.display());
        }
    }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, out);
        } else if matches!(path.extension().and_then(|e| e.to_str()), Some("kt" | "kts")) {
            out.push(path);
        }
    }
}
