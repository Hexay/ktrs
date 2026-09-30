#![allow(dead_code)]

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::SyntaxKind;

pub fn ast(text: &str) -> Ast {
    Ast::from_parse(&parse_file(text, FileKind::Source))
}

/// The `nth` node of `kind` in preorder.
pub fn find(ast: &Ast, kind: SyntaxKind, nth: usize) -> NodeId {
    ast.preorder(ast.root()).filter(|&n| ast.element_type(n) == kind).nth(nth).expect("no such node")
}

pub fn kinds(ast: &Ast, parent: NodeId) -> Vec<SyntaxKind> {
    let mut out = Vec::new();
    ast.get_children(parent, &mut out);
    out.into_iter().map(|c| ast.element_type(c)).collect()
}

pub fn root_text(ast: &Ast) -> String {
    ast.text(ast.root())
}
