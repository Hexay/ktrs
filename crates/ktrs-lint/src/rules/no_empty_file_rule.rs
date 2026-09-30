//! Port of ktlint-ruleset-standard `NoEmptyFileRule.kt`.

use ktrs_ast::psi::KtFile;
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{IMPORT_LIST, PACKAGE_DIRECTIVE, SCRIPT};

use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::kotlin_string::{is_blank, substring_after_last};

pub struct NoEmptyFileRule;

impl RuleV2 for NoEmptyFileRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-empty-file")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.is_root(node) && is_empty_file(ast, node) {
            emit(ast, 0, &format!("File '{}' should not be empty", get_file_name(ast, node)), false);
        }
    }
}

fn get_file_name(ast: &Ast, node: NodeId) -> String {
    let name = KtFile::of(ast, node).virtual_file_name(ast).replace('\\', "/"); // Ensure compatibility with Windows OS
    substring_after_last(&name, "/").to_owned()
}

fn is_empty_file(ast: &Ast, node: NodeId) -> bool {
    !ast.children(node).any(|it| {
        ast.is_code(it)
            && ast.element_type(it) != PACKAGE_DIRECTIVE
            && ast.element_type(it) != IMPORT_LIST
            && !(ast.element_type(it) == SCRIPT && is_blank(&ast.text(it)))
    })
}
