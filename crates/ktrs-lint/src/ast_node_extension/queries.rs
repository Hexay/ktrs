//! `ASTNodeExtension.kt` from `afterCodeSibling` to the end (bar `hasModifier`, `replaceWith`, `remove`):
//! sibling-type tests, recursive search, PSI-type checks and the max-line-length suppression lookup.

use ktrs_ast::psi::{self, DECLARATION_TYPES};
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{self, *};

use super::AstNodeExtension;
use crate::editorconfig::KtlintVersion;
use crate::engine::verifying_shortcuts;

const MAX_LINE_LENGTH_SUPPRESSION_ID: &str = "ktlint:standard:max-line-length";

pub trait AstNodeQueries {
    fn after_code_sibling(&self, n: NodeId, after_element_type: SyntaxKind) -> bool;
    fn before_code_sibling(&self, n: NodeId, before_element_type: SyntaxKind) -> bool;
    fn between_code_siblings(&self, n: NodeId, after_element_type: SyntaxKind, before_element_type: SyntaxKind) -> bool;
    fn find_child_by_type_recursively(&self, n: NodeId, element_type: SyntaxKind) -> Option<NodeId>;
    fn end_offset(&self, n: NodeId) -> usize;
    fn is_kt_annotated(&self, n: NodeId) -> bool;
    /// The jar's (Kotlin 2.4.10) `KtTokenSets.DECLARATION_TYPES` lacks the `DESTRUCTURING_DECLARATION` of our 2.4.20 pin
    /// (`tests/data/extension.jvm.txt`).
    fn is_declaration(&self, n: impl Into<Option<NodeId>>) -> bool;
    fn has_no_max_line_length_suppression(&self, n: NodeId) -> bool;
    /// ktlint 1.8 has no such check (#3255 added it to every "line too long" test), so it always holds.
    fn has_no_max_line_length_suppression_in(&self, n: NodeId, ktlint_version: KtlintVersion) -> bool {
        ktlint_version.is_1_8() || self.has_no_max_line_length_suppression(n)
    }
}

impl AstNodeQueries for Ast {
    fn after_code_sibling(&self, n: NodeId, after_element_type: SyntaxKind) -> bool {
        self.prev_sibling_matching(n, |it| self.is_code(it) && self.element_type(it) == after_element_type).is_some()
    }

    fn before_code_sibling(&self, n: NodeId, before_element_type: SyntaxKind) -> bool {
        self.next_sibling_matching(n, |it| self.is_code(it) && self.element_type(it) == before_element_type).is_some()
    }

    fn between_code_siblings(&self, n: NodeId, after_element_type: SyntaxKind, before_element_type: SyntaxKind) -> bool {
        self.after_code_sibling(n, after_element_type) && self.before_code_sibling(n, before_element_type)
    }

    fn find_child_by_type_recursively(&self, n: NodeId, element_type: SyntaxKind) -> Option<NodeId> {
        self.recursive_children(n).find(|&it| self.element_type(it) == element_type)
    }

    fn end_offset(&self, n: NodeId) -> usize {
        self.start_offset(n) + self.text_length(n)
    }

    /// `isKtAnnotated` (and `isPsiType<KtAnnotated>()`): see [`psi::is_kt_annotated`].
    fn is_kt_annotated(&self, n: NodeId) -> bool {
        psi::is_kt_annotated(self, n)
    }

    fn is_declaration(&self, n: impl Into<Option<NodeId>>) -> bool {
        n.into().is_some_and(|n| DECLARATION_TYPES.contains(self.element_type(n)) && self.element_type(n) != DESTRUCTURING_DECLARATION)
    }

    fn has_no_max_line_length_suppression(&self, n: NodeId) -> bool {
        let has_no_suppression = || {
            !is_annotated_with_max_line_length_suppression(self, n)
                && !self.parents(n).any(|it| is_annotated_with_max_line_length_suppression(self, it))
        };
        // A suppression needs a leaf with the rule id, which most files never have.
        if !self.allocated_leaf_text_contains(MAX_LINE_LENGTH_SUPPRESSION_ID) {
            assert!(!verifying_shortcuts() || has_no_suppression(), "max-line-length suppression gate missed a suppression");
            return true;
        }
        has_no_suppression()
    }
}

fn is_annotated_with_max_line_length_suppression(ast: &Ast, n: NodeId) -> bool {
    (ast.element_type(n) == ANNOTATED_EXPRESSION && contains_max_line_length_suppression(ast, Some(n)))
        || contains_max_line_length_suppression(ast, ast.find_child_by_type(n, MODIFIER_LIST))
        || (ast.is_root(n) && contains_max_line_length_suppression(ast, ast.find_child_by_type(n, FILE_ANNOTATION_LIST)))
}

fn contains_max_line_length_suppression(ast: &Ast, n: Option<NodeId>) -> bool {
    n.and_then(|n| ast.find_child_by_type(n, ANNOTATION_ENTRY))
        .and_then(|entry| ast.find_child_by_type(entry, VALUE_ARGUMENT_LIST))
        .is_some_and(|list| {
            ast.children(list).any(|it| {
                ast.element_type(it) == VALUE_ARGUMENT && ast.text_matches(it, "\"ktlint:standard:max-line-length\"")
            })
        })
}
