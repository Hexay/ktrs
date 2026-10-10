//! The facade over the compiler's parser fixtures (`testdata/kotlin/psi`): its dump, built with the public
//! navigation API only, must equal `ktrs_syntax::psi_dump` of the same parse for every fixture, and the
//! compiler's expected dump for the fixtures in ktrs-parser's ratchet. Then the invariants every tree must hold.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{env, fs};

use kt_syntax::{LineCol, Node, SourceFile, SyntaxKind, WalkEvent, parse, parse_script};
use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::psi_dump;

struct Fixture {
    name: String,
    file_name: String,
    text: String,
    expected: String,
    kind: FileKind,
}

fn fixtures() -> Vec<Fixture> {
    let psi_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/kotlin/psi");
    let mut sources = Vec::new();
    collect(&psi_dir, &mut sources);
    sources.sort();
    assert!(!sources.is_empty(), "no fixtures under {}", psi_dir.display());
    sources
        .iter()
        .map(|source| {
            let file_name = source.file_name().unwrap().to_str().unwrap().to_owned();
            // Upstream's test framework feeds the text with CRLF -> LF and trailing newlines stripped.
            let text = fs::read_to_string(source).unwrap().replace("\r\n", "\n");
            Fixture {
                name: source.strip_prefix(&psi_dir).unwrap().to_string_lossy().replace('\\', "/"),
                kind: FileKind::from_file_name(&file_name),
                file_name,
                text: text.trim_end_matches('\n').to_owned(),
                expected: fs::read_to_string(source.with_extension("txt")).unwrap().replace("\r\n", "\n"),
            }
        })
        .collect()
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, out);
        } else if matches!(path.extension().and_then(|e| e.to_str()), Some("kt" | "kts")) && path.with_extension("txt").is_file() {
            out.push(path);
        }
    }
}

fn parse_fixture(fixture: &Fixture) -> SourceFile {
    let parsed = match fixture.kind {
        FileKind::Source => parse(&fixture.text),
        FileKind::Script => parse_script(&fixture.text),
    };
    parsed.unwrap_or_else(|e| panic!("{}: {e}", fixture.name))
}

#[test]
fn dump_equals_psi_dump_and_the_compiler_fixtures() {
    let passing_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ktrs-parser/tests/passing.txt");
    let passing = fs::read_to_string(passing_path).unwrap();
    let passing: HashSet<&str> = passing.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    let fixtures = fixtures();
    let mut against_compiler = 0;
    for fixture in &fixtures {
        let file = parse_fixture(fixture);
        assert_eq!(file.text(), fixture.text.strip_prefix('\u{feff}').unwrap_or(&fixture.text), "{}", fixture.name);
        let dump = file.dump_named(&fixture.file_name);
        assert_eq!(dump, psi_dump(&parse_file(file.text(), fixture.kind), &fixture.file_name), "{}", fixture.name);
        if passing.contains(fixture.name.as_str()) {
            assert_eq!(dump.trim_end(), fixture.expected.trim_end(), "{}", fixture.name);
            against_compiler += 1;
        }
    }
    println!("{} fixtures equal psi_dump, {against_compiler} equal the compiler's dump", fixtures.len());
    assert_eq!(against_compiler, passing.len(), "every fixture of ktrs-parser's passing.txt was compared");
}

#[test]
fn every_fixture_tree_holds_the_invariants() {
    for fixture in &fixtures() {
        let file = parse_fixture(fixture);
        check_invariants(&file, &fixture.name);
    }
}

fn check_invariants(file: &SourceFile, name: &str) {
    let root = file.root();
    let text = file.text();
    assert_eq!(root.kind(), SyntaxKind::KT_FILE, "{name}");
    assert_eq!(root.range(), 0..text.len(), "{name}");
    assert_eq!(root.parent(), None, "{name}");
    assert_eq!(root.tokens().map(|t| t.text()).collect::<String>(), text, "{name}: tokens spell the text");
    assert_eq!(file.apply_edits([]).unwrap(), text, "{name}");

    let mut entered: Vec<Node> = Vec::new();
    let mut preorder = Vec::new();
    for event in root.preorder() {
        match event {
            WalkEvent::Enter(node) => {
                assert_eq!(node.parent(), entered.last().copied(), "{name}: {node:?}");
                entered.push(node);
                preorder.push(node);
            }
            WalkEvent::Leave(node) => assert_eq!(entered.pop(), Some(node), "{name}"),
        }
    }
    assert!(entered.is_empty(), "{name}");
    assert!(root.descendants().eq(preorder.iter().copied()), "{name}: descendants is the preorder");
    assert_eq!(root.descendants().len(), preorder.len(), "{name}");
    assert!(root.descendants().rev().eq(preorder.iter().rev().copied()), "{name}");

    let mut errors = file.errors();
    let mut prev_token: Option<Node> = None;
    for &node in &preorder {
        let range = node.range();
        assert_eq!(node.text(), &text[range.clone()], "{name}: {node:?}");
        check_children(node, name);
        assert!(node.ancestors().eq(std::iter::successors(node.parent(), |n| n.parent())), "{name}");
        assert_eq!(node.ancestors().last().unwrap_or(node), root, "{name}");
        if !range.is_empty() {
            assert_eq!(file.covering(range.clone()).map(|c| c.range()), Some(range.clone()), "{name}: {node:?}");
        }
        assert_eq!(SyntaxKind::from_name(node.kind().name()), Some(node.kind()), "{name}");
        assert_eq!(node.is_error(), node.kind() == SyntaxKind::ERROR_ELEMENT);
        if node.is_error() {
            let error = errors.next().unwrap_or_else(|| panic!("{name}: no error for {node:?}"));
            assert_eq!((error.node(), Some(error.message())), (node, node.error_message()), "{name}");
            assert_eq!(error.range(), range, "{name}");
        } else {
            assert_eq!(node.error_message(), None, "{name}");
        }
        if node.is_token() {
            assert_eq!(node.first_child(), None, "{name}");
            assert_eq!(node.prev_token(), prev_token, "{name}: {node:?}");
            if let Some(prev) = prev_token {
                assert_eq!(prev.next_token(), Some(node), "{name}: {prev:?}");
            }
            prev_token = Some(node);
            if !range.is_empty() {
                assert_eq!(file.token_at(range.start), Some(node), "{name}: {node:?}");
                assert_eq!(file.token_at(range.end - 1), Some(node), "{name}: {node:?}");
            }
            check_positions(file, range.start, name);
        } else {
            assert_eq!(node.first_token(), node.descendants().find(|n| n.is_token()), "{name}");
            assert_eq!(node.last_token(), node.descendants().rev().find(|n| n.is_token()), "{name}");
        }
    }
    assert_eq!(errors.next().map(|e| e.node()), None, "{name}: more errors than ERROR_ELEMENT nodes");
    assert_eq!(prev_token.and_then(|t| t.next_token()), None, "{name}");
    assert_eq!(file.token_at(text.len()), None, "{name}");
    assert_eq!(file.has_errors(), file.errors().len() > 0, "{name}");
    assert!(root.find_all(&[SyntaxKind::ERROR_ELEMENT]).eq(file.errors().map(|e| e.node())), "{name}");
    let functions = preorder.iter().copied().filter(|n| matches!(n.kind(), SyntaxKind::FUN | SyntaxKind::WHITE_SPACE));
    assert!(root.find_all(&[SyntaxKind::FUN, SyntaxKind::WHITE_SPACE]).eq(functions), "{name}: find_all");
    let comments = preorder.iter().copied().filter(|n| n.is_comment());
    assert!(root.comments().eq(comments), "{name}: comments");
}

fn check_children(node: Node, name: &str) {
    let children: Vec<Node> = node.children().collect();
    assert_eq!(node.first_child(), children.first().copied(), "{name}: {node:?}");
    assert_eq!(node.last_child(), children.last().copied(), "{name}: {node:?}");
    let mut offset = node.range().start;
    for (i, child) in children.iter().enumerate() {
        assert_eq!(child.parent(), Some(node), "{name}");
        assert_eq!(child.range().start, offset, "{name}: children tile {node:?}");
        offset = child.range().end;
        assert_eq!(child.prev_sibling(), i.checked_sub(1).map(|p| children[p]), "{name}");
        assert_eq!(child.next_sibling(), children.get(i + 1).copied(), "{name}");
        assert!(child.next_siblings().eq(children[i + 1..].iter().copied()), "{name}");
        assert!(child.prev_siblings().eq(children[..i].iter().rev().copied()), "{name}");
    }
    if !children.is_empty() {
        assert_eq!(offset, node.range().end, "{name}: children tile {node:?}");
    }
    assert!(node.child_nodes().eq(children.iter().copied().filter(|c| !c.is_token())), "{name}");
    assert!(node.leading_comments().all(|c| c.is_comment() && c.parent() == Some(node)), "{name}");
    if let Some(first) = children.first() {
        assert_eq!(node.child(first.kind()), Some(*first), "{name}");
    }
}

fn check_positions(file: &SourceFile, offset: usize, name: &str) {
    let at = file.line_col(offset).unwrap();
    assert_eq!(file.offset(at), Some(offset), "{name}: {at:?}");
    let line = file.line_range(at.line).unwrap();
    assert!(line.start <= offset && offset <= line.end, "{name}: {at:?}");
    assert_eq!(at.col as usize, offset - line.start, "{name}");
    let utf16 = file.line_col_utf16(offset).unwrap();
    assert_eq!(utf16, LineCol { line: at.line, col: file.text()[line.start..offset].encode_utf16().count() as u32 }, "{name}");
    assert_eq!(file.offset_utf16(utf16), Some(offset), "{name}: {utf16:?}");
}
