//! Port of ktlint-ruleset-standard `PackageNameRule.kt` (https://kotlinlang.org/docs/coding-conventions.html#naming-rules).

use std::sync::LazyLock;

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{DOT_QUALIFIED_EXPRESSION, PACKAGE_DIRECTIVE, REFERENCE_EXPRESSION};

use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::{KotlinRegex, reg_ex_ignoring_diacritics_and_strokes_on_letters};

pub struct PackageNameRule;

impl RuleV2 for PackageNameRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:package-name")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let expression = Some(node)
            .filter(|&it| ast.element_type(it) == PACKAGE_DIRECTIVE)
            .and_then(|it| ast.first_child_node(it))
            .and_then(|it| ast.next_code_sibling(it))
            .filter(|&it| matches!(ast.element_type(it), DOT_QUALIFIED_EXPRESSION | REFERENCE_EXPRESSION));
        if let Some(expression) = expression {
            let text = ast.text(expression);
            if text.contains('_') {
                emit(ast, ast.start_offset(expression), "Package name must not contain underscore", false);
            } else if !VALID_PACKAGE_NAME_REGEXP.matches(&text) {
                emit(ast, ast.start_offset(expression), "Package name contains a disallowed character", false);
            }
        }
    }
}

static VALID_PACKAGE_NAME_REGEXP: LazyLock<KotlinRegex> =
    LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("[a-z][a-zA-Z\\d]*(\\.[a-z][a-zA-Z\\d]*)*"));
