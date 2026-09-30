//! Port of ktlint-ruleset-standard `ModifierOrderRule.kt` (id `modifier-order`).

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{
    self, ABSTRACT_KEYWORD, ACTUAL_KEYWORD, ANNOTATION_ENTRY, ANNOTATION_KEYWORD, COMPANION_KEYWORD, CONST_KEYWORD,
    CONTEXT_PARAMETER_LIST, DATA_KEYWORD, ENUM_KEYWORD, EXPECT_KEYWORD, EXTERNAL_KEYWORD, FINAL_KEYWORD, INFIX_KEYWORD,
    INLINE_KEYWORD, INNER_KEYWORD, INTERNAL_KEYWORD, LATEINIT_KEYWORD, MODIFIER_LIST, OPEN_KEYWORD, OPERATOR_KEYWORD,
    OVERRIDE_KEYWORD, PRIVATE_KEYWORD, PROTECTED_KEYWORD, PUBLIC_KEYWORD, SEALED_KEYWORD, SUSPEND_KEYWORD, TAILREC_KEYWORD,
    VARARG_KEYWORD,
};

use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

// subset of ElementType.MODIFIER_KEYWORDS_ARRAY (+ annotations entries)
const ORDERED_MODIFIERS: [SyntaxKind; 27] = [
    ANNOTATION_ENTRY,
    CONTEXT_PARAMETER_LIST,
    PUBLIC_KEYWORD,
    PROTECTED_KEYWORD,
    PRIVATE_KEYWORD,
    INTERNAL_KEYWORD,
    EXPECT_KEYWORD,
    ACTUAL_KEYWORD,
    FINAL_KEYWORD,
    OPEN_KEYWORD,
    ABSTRACT_KEYWORD,
    SEALED_KEYWORD,
    CONST_KEYWORD,
    EXTERNAL_KEYWORD,
    OVERRIDE_KEYWORD,
    LATEINIT_KEYWORD,
    TAILREC_KEYWORD,
    VARARG_KEYWORD,
    SUSPEND_KEYWORD,
    INNER_KEYWORD,
    ENUM_KEYWORD,
    ANNOTATION_KEYWORD,
    COMPANION_KEYWORD,
    INLINE_KEYWORD,
    INFIX_KEYWORD,
    OPERATOR_KEYWORD,
    DATA_KEYWORD,
    // NOINLINE_KEYWORD, CROSSINLINE_KEYWORD, OUT_KEYWORD, IN_KEYWORD, REIFIED_KEYWORD
];
const TOKEN_SET: TokenSet = TokenSet::create(&ORDERED_MODIFIERS);

pub struct ModifierOrderRule;

impl RuleV2 for ModifierOrderRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:modifier-order")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != MODIFIER_LIST {
            return;
        }
        let mut modifier_arr = Vec::new();
        ast.get_children_filtered(node, TOKEN_SET, &mut modifier_arr);
        let mut sorted = modifier_arr.clone();
        // Stable, like Kotlin's `sortWith`.
        sorted.sort_by_key(|&it| ORDERED_MODIFIERS.iter().position(|&t| t == ast.element_type(it)));
        if modifier_arr != sorted {
            // Since annotations can be fairly lengthy and/or span multiple lines we are
            // squashing them into a single placeholder text to guarantee a single line output
            let squashed_annotations = squash_annotations(ast, &sorted).join(" ");
            let message = format!("Incorrect modifier order (should be \"{squashed_annotations}\")");
            let autocorrect = emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| ()).is_some();
            if autocorrect {
                for (i, &n) in modifier_arr.iter().enumerate() {
                    let clone = ast.clone(sorted[i]);
                    ast.replace_child(node, n, clone);
                }
            }
        }
    }
}

fn squash_annotations(ast: &Ast, sorted: &[NodeId]) -> Vec<String> {
    let non_annotation_modifiers: Vec<NodeId> = sorted.iter().copied().filter(|&it| ast.element_type(it) != ANNOTATION_ENTRY).collect();
    let texts = non_annotation_modifiers.iter().map(|&it| ast.text(it));
    if non_annotation_modifiers.len() != sorted.len() {
        std::iter::once("@Annotation...".to_owned()).chain(texts).collect()
    } else {
        texts.collect()
    }
}
