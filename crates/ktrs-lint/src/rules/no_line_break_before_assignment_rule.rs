//! Port of ktlint-ruleset-standard `NoLineBreakBeforeAssignmentRule.kt` (id `no-line-break-before-assignment`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{EQ, WHITE_SPACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct NoLineBreakBeforeAssignmentRule;

impl RuleV2 for NoLineBreakBeforeAssignmentRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-line-break-before-assignment")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == EQ {
            visit_equals(ast, node, emit);
        }
    }
}

fn visit_equals(ast: &mut Ast, assignment_node: NodeId, emit: &mut Emit<'_>) {
    let Some(unexpected_newline_before_assignment) =
        ast.prev_sibling(assignment_node).filter(|&it| ast.is_white_space_with_newline(it))
    else {
        return;
    };
    emit(ast, ast.start_offset(unexpected_newline_before_assignment), "Line break before assignment is not allowed", true)
        .if_autocorrect_allowed(|| {
            let parent = ast.parent(assignment_node).expect("NullPointerException: parent!!");
            // Insert assignment surrounded by whitespaces at new position
            let before = ast
                .siblings(assignment_node, false)
                .take_while(|&it| !ast.is_code(it))
                .last()
                .expect("NoSuchElementException: Sequence is empty.");
            if !ast.is_white_space(ast.prev_sibling(before)) {
                let white_space = ast.new_leaf(WHITE_SPACE, " ");
                ast.add_child(parent, white_space, Some(before));
            }
            let eq = ast.new_leaf(EQ, "=");
            ast.add_child(parent, eq, Some(before));
            if !ast.is_white_space(before) {
                let white_space = ast.new_leaf(WHITE_SPACE, " ");
                ast.add_child(parent, white_space, Some(before));
            }
            // Cleanup old assignment and whitespace after it. The indent before the old assignment is kept unchanged
            if let Some(it) = ast.next_sibling(assignment_node).filter(|&it| ast.is_white_space(it)) {
                ast.remove(it);
            }
            ast.remove(assignment_node);
        });
}
