//! Every `RuleV2` impl that overrides `after_visit_child_nodes` must say so in `visits_after_child_nodes`,
//! or the engine skips its hook (and the reverse, which would only cost time).

use std::fs;
use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The body of each `impl RuleV2 for X { ... }` block, by brace matching from its opening brace.
fn rule_impls(source: &str) -> Vec<(&str, &str)> {
    let mut impls = Vec::new();
    for (start, _) in source.match_indices("impl RuleV2 for ") {
        let header = &source[start..];
        let name = header["impl RuleV2 for ".len()..].split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap();
        let open = start + header.find('{').unwrap();
        let mut depth = 0;
        let end = source[open..]
            .char_indices()
            .find_map(|(i, c)| {
                depth += match c {
                    '{' => 1,
                    '}' => -1,
                    _ => 0,
                };
                (depth == 0).then_some(open + i)
            })
            .unwrap();
        impls.push((name, &source[open..end]));
    }
    impls
}

#[test]
fn visits_after_child_nodes_matches_the_overrides() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for dir in ["src", "tests", "examples"] {
        rust_files(&root.join(dir), &mut files);
    }
    let mut checked = 0;
    for file in files.into_iter().filter(|f| !f.ends_with("rule_hooks.rs")) {
        let source = fs::read_to_string(&file).unwrap();
        for (name, body) in rule_impls(&source) {
            let overrides = body.contains("fn after_visit_child_nodes(");
            let declares = body.contains("fn visits_after_child_nodes(");
            assert_eq!(overrides, declares, "{name} in {}", file.display());
            checked += 1;
        }
    }
    assert!(checked > 100, "only {checked} rule impls found");
}
