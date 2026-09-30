//! Seeding an unedited arena must reproduce the parser's tree exactly: same dump as `psi_dump` on every
//! parser fixture, same offsets, same text.

use std::path::{Path, PathBuf};
use std::{env, fs};

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::{Tree, psi_dump};

#[test]
fn seeded_dump_equals_psi_dump_on_parser_fixtures() {
    let psi_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/kotlin/psi");
    let mut fixtures = Vec::new();
    collect(&psi_dir, &mut fixtures);
    assert!(fixtures.len() > 400, "fixtures missing under {}", psi_dir.display());
    let mut failures = Vec::new();
    for source in &fixtures {
        let name = source.file_name().unwrap().to_str().unwrap();
        let text = fs::read_to_string(source).unwrap().replace("\r\n", "\n");
        let parse = parse_file(text.trim_end_matches('\n'), FileKind::from_file_name(name));
        let ast = Ast::from_parse(&parse);
        if ast.psi_to_string(ast.root(), name) != psi_dump(&parse, name) {
            failures.push(format!("dump {name}"));
        }
        if ast.text(ast.root()) != parse.tree.text() {
            failures.push(format!("text {name}"));
        }
        if let Some(e) = first_offset_difference(&ast, &parse.tree) {
            failures.push(format!("offset {name} element {e}"));
        }
    }
    assert!(failures.is_empty(), "{} of {} fixtures differ:\n{}", failures.len(), fixtures.len(), failures.join("\n"));
}

/// Walks the arena in preorder next to the tree's preorder indices.
fn first_offset_difference(ast: &Ast, tree: &Tree) -> Option<u32> {
    let mut node = Some(ast.root());
    let mut e = 0;
    while let Some(n) = node {
        let range = tree.text_range(e);
        if ast.start_offset(n) != usize::from(range.start()) || ast.text_length(n) != usize::from(range.len()) {
            return Some(e);
        }
        e += 1;
        node = ast.first_child_node(n).or_else(|| next_up(ast, n));
    }
    (e as usize != tree.len()).then_some(e)
}

fn next_up(ast: &Ast, mut n: NodeId) -> Option<NodeId> {
    loop {
        if let Some(next) = ast.tree_next(n) {
            return Some(next);
        }
        n = ast.tree_parent(n)?;
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
