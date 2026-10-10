//! The public API on small hand-written inputs. Tree-wide invariants are checked on the compiler's fixtures in
//! `fixtures.rs`; nesting limits in `nesting.rs`.

use std::collections::HashSet;

use kt_syntax::{EditError, KOTLIN_VERSION, LineCol, Node, ParseError, SourceFile, SyntaxKind, WalkEvent, parse, parse_script};

const SOURCE: &str = "\
package demo

// about Greeter
/**
 * Greets.
 * @param name who
 */
class Greeter(val name: String) {
    fun greet() = println(\"hi $name\") // trailing
}
";

fn kinds(nodes: impl Iterator<Item = Node<'static>>) -> Vec<&'static str> {
    nodes.map(|node| node.kind().name()).collect()
}

fn leak(text: &str) -> &'static SourceFile {
    Box::leak(Box::new(parse(text).unwrap()))
}

fn first(file: &SourceFile, kind: SyntaxKind) -> Node<'_> {
    file.root().find_all(&[kind]).next().unwrap_or_else(|| panic!("no {kind}"))
}

#[test]
fn source_file_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SourceFile>();
    assert_send_sync::<Node<'static>>();
    let file = parse(SOURCE).unwrap();
    let functions = std::thread::scope(|scope| {
        let handle = scope.spawn(|| file.root().find_all(&[SyntaxKind::FUN]).count());
        handle.join().unwrap()
    });
    assert_eq!(functions, 1);
}

#[test]
fn kotlin_version_is_the_pin() {
    assert_eq!(KOTLIN_VERSION, "2.4.20");
}

#[test]
fn bom_and_crlf_are_normalized_at_the_entry() {
    let file = parse("\u{feff}val a = 1\r\nval b = 2\r\n").unwrap();
    assert_eq!(file.text(), "val a = 1\nval b = 2\n");
    assert!(file.had_bom() && file.had_crlf());
    assert_eq!(file.root().range(), 0..file.text().len());
    assert_eq!(file.line_count(), 3);
    assert_eq!(file.dump(), parse("val a = 1\nval b = 2\n").unwrap().dump());

    let plain = parse("val a = 1\n").unwrap();
    assert!(!plain.had_bom() && !plain.had_crlf());

    let lone_cr = parse("val a = 1\rval b = 2").unwrap();
    assert_eq!(lone_cr.text(), "val a = 1\rval b = 2");
    assert!(!lone_cr.had_crlf());
    assert_eq!(lone_cr.line_count(), 1);
}

#[test]
fn scripts_allow_top_level_statements() {
    let text = "println(1)\n";
    let script = parse_script(text).unwrap();
    assert!(script.is_script() && !script.has_errors());
    assert!(script.root().find_all(&[SyntaxKind::SCRIPT]).next().is_some());
    let source = parse(text).unwrap();
    assert!(!source.is_script() && source.has_errors());
}

#[test]
fn empty_input_is_a_file_without_tokens() {
    let file = parse("").unwrap();
    let root = file.root();
    assert_eq!((root.kind(), root.range(), root.text()), (SyntaxKind::KT_FILE, 0..0, ""));
    assert_eq!(root.tokens().count(), 0);
    assert_eq!((root.first_token(), root.next_token(), root.prev_token()), (None, None, None));
    assert_eq!(file.token_at(0), None);
    assert_eq!(file.covering(0..0), Some(root));
    assert_eq!(file.covering(0..1), None);
    assert_eq!(file.line_col(0), Some(LineCol { line: 0, col: 0 }));
    assert!(!file.has_errors());
}

#[test]
fn nodes_and_tokens() {
    let file = leak(SOURCE);
    let class = first(file, SyntaxKind::CLASS);
    assert!(!class.is_token() && !class.is_trivia() && !class.is_error());
    assert_eq!(class.range().end, SOURCE.len() - 1);
    assert!(class.text().starts_with("/**") && class.text().ends_with('}'));
    assert_eq!(kinds(class.child_nodes()), ["DOC_COMMENT", "PRIMARY_CONSTRUCTOR", "CLASS_BODY"]);
    assert_eq!(
        kinds(class.children()),
        ["DOC_COMMENT", "WHITE_SPACE", "CLASS_KEYWORD", "WHITE_SPACE", "IDENTIFIER", "PRIMARY_CONSTRUCTOR", "WHITE_SPACE", "CLASS_BODY"]
    );

    let keyword = class.child(SyntaxKind::CLASS_KEYWORD).unwrap();
    assert!(keyword.is_token() && !keyword.is_trivia());
    assert_eq!((keyword.text(), keyword.kind().keyword_text()), ("class", Some("class")));
    assert_eq!(keyword.children().count(), 0);
    assert_eq!(keyword.tokens().collect::<Vec<_>>(), [keyword]);
    assert_eq!((keyword.first_token(), keyword.last_token()), (Some(keyword), Some(keyword)));
    assert_eq!(keyword.file().text(), SOURCE);

    let space = keyword.next_sibling().unwrap();
    assert!(space.is_whitespace() && space.is_trivia() && !space.is_comment());
    assert_eq!(class.child(SyntaxKind::FUN), None, "child looks at direct children only");
    assert_eq!(format!("{keyword:?}"), format!("CLASS_KEYWORD@{}..{}", keyword.range().start, keyword.range().end));
}

#[test]
fn navigation() {
    let file = leak(SOURCE);
    let function = first(file, SyntaxKind::FUN);
    let name = function.child(SyntaxKind::IDENTIFIER).unwrap();
    assert_eq!(name.text(), "greet");
    assert_eq!(name.parent(), Some(function));
    assert_eq!(kinds(name.ancestors()), ["FUN", "CLASS_BODY", "CLASS", "KT_FILE"]);
    assert_eq!(kinds(name.prev_siblings()), ["WHITE_SPACE", "FUN_KEYWORD"]);
    assert_eq!(kinds(name.next_siblings().take(2)), ["VALUE_PARAMETER_LIST", "WHITE_SPACE"]);
    assert_eq!(function.first_child().unwrap().text(), "fun");
    assert_eq!(function.last_child().unwrap().kind(), SyntaxKind::EOL_COMMENT, "Kotlin binds the trailing comment to the function");
    assert_eq!(function.first_token().unwrap().text(), "fun");
    assert_eq!(function.last_token().unwrap().text(), "// trailing");
    assert_eq!(function.next_token().unwrap().text(), "\n");
    assert_eq!(function.prev_token().unwrap().text(), "\n    ");
    assert_eq!(name.next_token().unwrap().text(), "(");
    assert_eq!(function.tokens().map(|t| t.text()).collect::<String>(), function.text());
    assert_eq!(function.descendants().next(), Some(function));
    assert_eq!(function.descendants().len(), function.descendants().count());
    assert!(function.descendants().all(|node| node == function || node.ancestors().any(|a| a == function)));
}

#[test]
fn node_identity_is_per_file() {
    let (a, b) = (parse(SOURCE).unwrap(), parse(SOURCE).unwrap());
    assert_eq!(a.root(), a.root());
    assert_ne!(a.root(), b.root());
    assert_ne!(a.root(), a.root().first_child().unwrap());
    let set: HashSet<Node> = a.root().descendants().chain(a.root().descendants()).collect();
    assert_eq!(set.len(), a.root().descendants().len());
}

#[test]
fn preorder_enters_and_leaves_and_skips() {
    let file = parse("fun f() { g() }").unwrap();
    let mut depth = 0;
    let mut deepest = 0;
    for event in file.root().preorder() {
        match event {
            WalkEvent::Enter(_) => depth += 1,
            WalkEvent::Leave(_) => depth -= 1,
        }
        deepest = deepest.max(depth);
    }
    assert_eq!(depth, 0);
    assert!(deepest > 5);

    let mut walk = file.root().preorder();
    let mut seen = Vec::new();
    while let Some(event) = walk.next() {
        match event {
            WalkEvent::Enter(node) if node.kind() == SyntaxKind::BLOCK => {
                seen.push("enter BLOCK");
                walk.skip_subtree();
            }
            WalkEvent::Leave(node) if node.kind() == SyntaxKind::BLOCK => {
                seen.push("leave BLOCK");
                walk.skip_subtree();
            }
            WalkEvent::Enter(node) if node.kind() == SyntaxKind::CALL_EXPRESSION => seen.push("call"),
            WalkEvent::Leave(node) if node.kind() == SyntaxKind::KT_FILE => seen.push("leave file"),
            _ => {}
        }
    }
    assert_eq!(seen, ["enter BLOCK", "leave BLOCK", "leave file"]);

    let token = file.root().first_token().unwrap();
    assert_eq!(token.preorder().collect::<Vec<_>>(), [WalkEvent::Enter(token), WalkEvent::Leave(token)]);
}

#[test]
fn find_all_with_few_and_many_kinds() {
    let file = leak(SOURCE);
    let root = file.root();
    assert_eq!(kinds(root.find_all(&[])), [] as [&str; 0]);
    assert_eq!(kinds(root.find_all(&[SyntaxKind::KT_FILE])), ["KT_FILE"], "the node itself is included");
    assert_eq!(kinds(root.find_all(&[SyntaxKind::FUN, SyntaxKind::CLASS])), ["CLASS", "FUN"]);
    assert_eq!(root.find_all(&[SyntaxKind::IDENTIFIER]).map(|n| n.text()).collect::<Vec<_>>(), ["demo", "name", "Greeter", "name", "String", "greet", "println", "name"], "KDoc names included");
    let many = [
        SyntaxKind::FUN,
        SyntaxKind::CLASS,
        SyntaxKind::PROPERTY,
        SyntaxKind::OBJECT_DECLARATION,
        SyntaxKind::TYPEALIAS,
        SyntaxKind::VALUE_PARAMETER,
        SyntaxKind::PACKAGE_DIRECTIVE,
    ];
    assert_eq!(kinds(root.find_all(&many)), ["PACKAGE_DIRECTIVE", "CLASS", "VALUE_PARAMETER", "FUN"]);
    let class = first(file, SyntaxKind::CLASS);
    assert_eq!(class.find_all(&[SyntaxKind::PACKAGE_DIRECTIVE]).count(), 0, "only the subtree");
}

#[test]
fn positions() {
    let file = parse(SOURCE).unwrap();
    let name = first(&file, SyntaxKind::FUN).child(SyntaxKind::IDENTIFIER).unwrap();
    let at = file.line_col(name.range().start).unwrap();
    assert_eq!(at, LineCol { line: 8, col: 8 });
    assert_eq!(file.offset(at), Some(name.range().start));
    assert_eq!(file.line_col_utf16(name.range().start), Some(at));
    assert_eq!(&file.text()[file.line_range(0).unwrap()], "package demo");
    assert_eq!(file.line_range(99), None);
    assert_eq!(file.line_col(SOURCE.len() + 1), None);

    let wide = parse("val s = \"π𝄞\"; val t = 1").unwrap();
    let t = wide.root().find_all(&[SyntaxKind::PROPERTY]).nth(1).unwrap();
    let (bytes, units) = (wide.line_col(t.range().start).unwrap(), wide.line_col_utf16(t.range().start).unwrap());
    assert_eq!((bytes.col, units.col), (18, 15));
    assert_eq!(wide.offset_utf16(units), Some(t.range().start));
}

#[test]
fn token_at_and_covering() {
    let file = parse(SOURCE).unwrap();
    let name = first(&file, SyntaxKind::FUN).child(SyntaxKind::IDENTIFIER).unwrap();
    let range = name.range();
    assert_eq!(file.token_at(range.start), Some(name));
    assert_eq!(file.token_at(range.end - 1), Some(name));
    assert_eq!(file.token_at(range.end).unwrap().text(), "(");
    assert_eq!(file.token_at(SOURCE.len()), None);
    assert_eq!(file.covering(range.clone()), Some(name));
    assert_eq!(file.covering(range.start..range.start), Some(name));
    assert_eq!(file.covering(range.start + 1..range.end + 1).unwrap().kind(), SyntaxKind::FUN);
    assert_eq!(file.covering(0..SOURCE.len()), Some(file.root()));
    assert_eq!(file.covering(SOURCE.len()..SOURCE.len()), Some(file.root()));
    assert_eq!(file.covering(0..SOURCE.len() + 1), None);
    #[allow(clippy::reversed_empty_ranges)]
    let reversed = 5..2;
    assert_eq!(file.covering(reversed), None);
}

#[test]
fn syntax_errors_are_nodes_with_messages() {
    let file = parse("fun f( {}\nval = 1\n").unwrap();
    assert!(file.has_errors());
    let errors: Vec<_> = file.errors().collect();
    assert!(errors.len() >= 2);
    for error in &errors {
        assert!(error.node().is_error());
        assert_eq!(error.node().error_message(), Some(error.message()));
        assert_eq!(error.range(), error.node().range());
        assert!(!error.message().is_empty());
        let at = file.line_col(error.range().start).unwrap();
        assert_eq!(error.to_string(), format!("{}:{}: {}", at.line + 1, at.col + 1, error.message()));
    }
    assert!(errors.windows(2).all(|pair| pair[0].range().start <= pair[1].range().start), "source order");
    assert!(file.dump().contains(&format!("PsiErrorElement:{}", errors[0].message())));
    assert_eq!(file.root().text(), file.text());

    let clean = parse(SOURCE).unwrap();
    assert!(!clean.has_errors());
    assert_eq!(clean.errors().count(), 0);
    assert_eq!(clean.root().error_message(), None);
}

#[test]
fn input_whose_tokens_kotlin_drops_is_an_error() {
    // research/24, finding 4: the lambda reparse stops before `)`.
    let text = "{fun<)]<T:@( {})";
    let expected = ParseError::TokensNotInserted { tokens: vec!["RPAR".to_owned()] };
    assert_eq!(parse(text).unwrap_err(), expected);
    assert_eq!(parse_script(text).unwrap_err(), expected);
    assert_eq!(expected.to_string(), "Tokens [RPAR] were not inserted into the tree. Language: kotlin");
}

#[test]
fn comments_and_kdoc() {
    let file = leak(SOURCE);
    let root = file.root();
    assert_eq!(kinds(root.comments()), ["EOL_COMMENT", "DOC_COMMENT", "EOL_COMMENT"]);
    assert!(root.comments().all(|comment| comment.is_comment() && comment.is_trivia()));

    let class = first(file, SyntaxKind::CLASS);
    let kdoc = class.doc_comment().unwrap();
    assert!(!kdoc.is_token(), "KDoc is a node");
    assert!(kdoc.text().starts_with("/**") && kdoc.text().ends_with("*/"));
    let tag = kdoc.find_all(&[SyntaxKind::KDOC_TAG]).next().unwrap();
    assert_eq!(tag.child(SyntaxKind::KDOC_TAG_NAME).unwrap().text(), "@param");
    assert_eq!(first(file, SyntaxKind::KDOC_NAME).text(), "name");
    assert_eq!(kinds(class.leading_comments()), ["DOC_COMMENT"]);
    assert_eq!(class.comments().count(), 2);

    let function = first(file, SyntaxKind::FUN);
    assert_eq!(function.doc_comment(), None);
    assert_eq!(function.leading_comments().count(), 0);
    let trailing = root.comments().last().unwrap();
    assert_eq!(trailing.text(), "// trailing");
    assert!(trailing.is_token());

    let bound = leak("// one\n/* two */\nfun f() {}\n");
    assert_eq!(first(bound, SyntaxKind::FUN).leading_comments().map(|c| c.text()).collect::<Vec<_>>(), ["// one", "/* two */"]);
    let shebang = Box::leak(Box::new(parse_script("#!/usr/bin/env kotlin\nprintln(1)\n").unwrap()));
    assert_eq!(kinds(shebang.root().comments()), ["SHEBANG_COMMENT"]);
}

#[test]
fn edits_from_nodes() {
    let file = parse("fun f() { old(1); old(2) }\n").unwrap();
    let callees: Vec<Node> = file.root().find_all(&[SyntaxKind::CALL_EXPRESSION]).filter_map(|call| call.first_child()).collect();
    let function = first(&file, SyntaxKind::FUN);
    let mut edits: Vec<_> = callees.iter().map(|callee| callee.replace("new")).collect();
    edits.push(function.insert_before("/** Doc. */\n"));
    edits.push(function.insert_after("\nfun g() {}"));
    assert_eq!(file.apply_edits(edits).unwrap(), "/** Doc. */\nfun f() { new(1); new(2) }\nfun g() {}\n");

    let first_call = callees[0].parent().unwrap();
    assert_eq!(file.apply_edits([first_call.remove()]).unwrap(), "fun f() { ; old(2) }\n");
    let overlapping = file.apply_edits([function.remove(), callees[0].replace("x")]);
    assert_eq!(overlapping, Err(EditError::Overlapping { first: function.range(), second: callees[0].range() }));

    let reparsed = parse(&file.apply_edits([callees[1].replace("other")]).unwrap()).unwrap();
    assert_eq!(reparsed.root().find_all(&[SyntaxKind::CALL_EXPRESSION]).nth(1).unwrap().text(), "other(2)");
}
