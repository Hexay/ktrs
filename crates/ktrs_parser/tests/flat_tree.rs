//! The flat `Tree` must hold exactly the rowan tree: their dumps agree on every fixture, and a
//! cached re-parse (which splices cached subtrees) builds the same `Tree` as a fresh one.
//! `FLAT_TREE_CORPUS=<dir>` checks every .kt/.kts under that directory as well.
//! TODO: delete with the rowan copy in `Parse` (research/06-tree-library.md, step 4).

use std::path::{Path, PathBuf};
use std::{env, fs};

use ktrs_parser::{ChameleonCache, FileKind, parse_file, parse_file_cached};
use ktrs_syntax::{psi_dump, psi_dump_green};

#[test]
fn flat_tree_equals_green_tree() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/kotlin/psi");
    let mut files = Vec::new();
    collect(&dir, &mut files);
    assert!(!files.is_empty(), "no fixtures under {}; run tools/sync-kotlin.sh", dir.display());
    if let Some(corpus) = env::var_os("FLAT_TREE_CORPUS") {
        collect(Path::new(&corpus), &mut files);
    }

    let mut cache = ChameleonCache::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap().replace("\r\n", "\n");
        let kind = FileKind::from_file_name(&file.to_string_lossy());
        let parse = parse_file(&text, kind);
        assert_eq!(parse.tree.text(), text, "{}", file.display());
        assert_eq!(psi_dump(&parse, ""), psi_dump_green(&parse, ""), "{}", file.display());
        for _ in 0..2 {
            assert_eq!(parse_file_cached(&text, kind, &mut cache).tree, parse.tree, "cached: {}", file.display());
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
