//! Port of the `KtLintDirective` half of ktlint-rule-engine `internal/rules/KtlintSuppressionRule.kt`:
//! parsing `ktlint-disable`/`ktlint-enable` comments and matching them up.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    BLOCK_COMMENT, CLASS, EOL_COMMENT, FUN, PROPERTY, TYPE_ARGUMENT_LIST, TYPE_PARAMETER,
    TYPE_PARAMETER_LIST, TYPE_PROJECTION, VALUE_ARGUMENT, VALUE_ARGUMENT_LIST, VALUE_PARAMETER,
    VALUE_PARAMETER_LIST,
};

use crate::ast_node_extension::AstNodeExtension;
use crate::engine::ktlint_suppression::{
    KTLINT_SUPPRESSION_ID_ALL_RULES, is_top_level, qualified_rule_id_string,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum KtlintDirectiveType {
    KtlintDisable,
    KtlintEnable,
}

impl KtlintDirectiveType {
    pub(super) fn id(self) -> &'static str {
        match self {
            KtlintDirectiveType::KtlintDisable => "ktlint-disable",
            KtlintDirectiveType::KtlintEnable => "ktlint-enable",
        }
    }
}

/// `SuppressionIdChange`; `offset_original_rule_id` is `indexOf` of the *qualified* id in the raw ids,
/// so -1 unless the comment spelled it out.
#[derive(Clone, Debug)]
pub(super) enum SuppressionIdChange {
    ValidSuppressionId {
        suppression_id: String,
    },
    InvalidSuppressionId {
        original_rule_id: String,
        offset_original_rule_id: isize,
    },
}

pub(super) struct KtLintDirective<'v> {
    pub(super) node: NodeId,
    rule_id_validator: &'v dyn Fn(&str) -> bool,
    pub(super) ktlint_directive_type: KtlintDirectiveType,
    ktlint_directives: String,
    pub(super) suppression_id_changes: Vec<SuppressionIdChange>,
    /// `node.startOffset + node.text.indexOf(type.id)`.
    pub(super) offset: usize,
}

impl KtLintDirective<'_> {
    pub(super) fn has_no_matching_ktlint_enable_directive(&self, ast: &Ast) -> bool {
        self.require_block_disable(ast);
        if should_be_converted_to_file_annotation(ast, self.node) {
            false
        } else {
            self.find_matching_ktlint_enable_directive(ast).is_none()
        }
    }

    fn find_matching_ktlint_enable_directive(&self, ast: &Ast) -> Option<NodeId> {
        let start = if is_suppressible_declaration(ast, self.node) {
            ast.parent(self.node)?
        } else {
            self.node
        };
        let mut sibling = ast.tree_next(start);
        while let Some(node) = sibling {
            let matches = ktlint_directive_or_null(ast, node, self.rule_id_validator)
                .filter(|it| it.ktlint_directive_type == KtlintDirectiveType::KtlintEnable)
                .is_some_and(|it| it.ktlint_directives == self.ktlint_directives);
            if matches {
                return Some(node);
            }
            sibling = ast.tree_next(node);
        }
        None
    }

    pub(super) fn should_be_promoted_to_parent_declaration(&self, ast: &Ast) -> bool {
        self.require_block_disable(ast);
        if should_be_converted_to_file_annotation(ast, self.node) {
            return false;
        }
        if is_suppressible_declaration(ast, self.node) {
            // Matched by the declaration's next sibling, the directive belongs to that declaration only.
            return self
                .find_matching_ktlint_enable_directive(ast)
                .is_some_and(|matching| {
                    Some(matching)
                        != ast.parent(self.node).and_then(|p| {
                            ast.next_sibling_matching(p, |it| !ast.is_white_space(it))
                        })
                });
        }
        self.surrounds_multiple_list_elements(ast)
    }

    fn surrounds_multiple_list_elements(&self, ast: &Ast) -> bool {
        self.require_block_disable(ast);
        let in_list = ast.parent(self.node).is_some_and(|p| {
            matches!(
                ast.element_type(p),
                TYPE_ARGUMENT_LIST
                    | TYPE_PARAMETER_LIST
                    | VALUE_ARGUMENT_LIST
                    | VALUE_PARAMETER_LIST
            )
        });
        if !in_list {
            return false;
        }
        let Some(matching) = self.find_matching_ktlint_enable_directive(ast) else {
            return false;
        };
        let mut count = 0;
        let mut sibling = ast.tree_prev(matching);
        while let Some(node) = sibling.filter(|&it| it != self.node) {
            if matches!(
                ast.element_type(node),
                TYPE_PROJECTION | TYPE_PARAMETER | VALUE_ARGUMENT | VALUE_PARAMETER
            ) {
                count += 1;
            }
            sibling = ast.tree_prev(node);
        }
        count > 1
    }

    fn require_block_disable(&self, ast: &Ast) {
        assert!(
            self.ktlint_directive_type == KtlintDirectiveType::KtlintDisable
                && ast.element_type(self.node) == BLOCK_COMMENT,
            "IllegalArgumentException: Failed requirement."
        );
    }
}

pub(super) fn ktlint_directive_or_null<'v>(
    ast: &Ast,
    node: NodeId,
    rule_id_validator: &'v dyn Fn(&str) -> bool,
) -> Option<KtLintDirective<'v>> {
    let text = match ast.element_type(node) {
        EOL_COMMENT | BLOCK_COMMENT => ast.text(node),
        _ => return None,
    };
    let ktlint_directive_string = if ast.element_type(node) == EOL_COMMENT {
        text.strip_prefix("//").unwrap_or(&text).trim()
    } else {
        let t = text.strip_prefix("/*").unwrap_or(&text);
        t.strip_suffix("*/").unwrap_or(t).trim()
    };
    let ktlint_directive_type = [
        KtlintDirectiveType::KtlintDisable,
        KtlintDirectiveType::KtlintEnable,
    ]
    .into_iter()
    .find(|t| ktlint_directive_string.starts_with(t.id()))?;
    let rule_ids = &ktlint_directive_string[ktlint_directive_type.id().len()..];
    let suppression_id_changes = to_suppression_id_changes(rule_ids, rule_id_validator);
    let offset = ast.start_offset(node)
        + text
            .find(ktlint_directive_type.id())
            .expect("the directive is in the text");
    Some(KtLintDirective {
        node,
        rule_id_validator,
        ktlint_directive_type,
        ktlint_directives: rule_ids.to_owned(),
        suppression_id_changes,
        offset,
    })
}

/// `"ktlint-disable foo standard:bar"` -> the qualified ids `standard:foo`, `standard:bar`; none means all.
fn to_suppression_id_changes(
    rule_ids: &str,
    rule_id_validator: &dyn Fn(&str) -> bool,
) -> Vec<SuppressionIdChange> {
    let changes: Vec<SuppressionIdChange> = rule_ids
        .trim()
        .split(' ')
        .map(str::trim)
        .filter(|it| !it.is_empty())
        .map(qualified_rule_id_string)
        .map(|original_rule_id| {
            if rule_id_validator(&original_rule_id) {
                SuppressionIdChange::ValidSuppressionId {
                    suppression_id: original_rule_id,
                }
            } else {
                let offset_original_rule_id =
                    rule_ids.find(&original_rule_id).map_or(-1, |i| i as isize);
                SuppressionIdChange::InvalidSuppressionId {
                    original_rule_id,
                    offset_original_rule_id,
                }
            }
        })
        .collect();
    if changes.is_empty() {
        vec![SuppressionIdChange::ValidSuppressionId {
            suppression_id: KTLINT_SUPPRESSION_ID_ALL_RULES.to_owned(),
        }]
    } else {
        changes
    }
}

pub(super) fn should_be_converted_to_file_annotation(ast: &Ast, node: NodeId) -> bool {
    is_top_level(ast, node)
        || (ast.element_type(node) == BLOCK_COMMENT
            && is_suppressible_declaration(ast, node)
            && is_top_level(
                ast,
                ast.parent(node).expect("NullPointerException: parent!!"),
            ))
}

fn is_suppressible_declaration(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .is_some_and(|p| matches!(ast.element_type(p), CLASS | FUN | PROPERTY))
}
