//! Port of ktlint-ruleset-standard `AnnotationSpacingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{ANNOTATION_ENTRY, FILE_ANNOTATION_LIST, MODIFIER_LIST, WHITE_SPACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeQueries};
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[MODIFIER_LIST, FILE_ANNOTATION_LIST]);

const ERROR_MESSAGE: &str = "Annotations should occur immediately before the annotated construct";

/// Ensures annotations occur immediately prior to the annotated construct
///
/// https://kotlinlang.org/docs/reference/coding-conventions.html#annotation-formatting
pub struct AnnotationSpacingRule;

impl RuleV2 for AnnotationSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:annotation-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != MODIFIER_LIST && ast.element_type(node) != FILE_ANNOTATION_LIST {
            return;
        }

        let annotations: Vec<NodeId> = ast.children(node).filter(|&it| ast.element_type(it) == ANNOTATION_ENTRY).collect();
        if annotations.is_empty() {
            return;
        }

        // Join the nodes that immediately follow the annotations (whitespace), then add the final whitespace
        // if it's not a child of root. This happens when a new line separates the annotations from the annotated
        // construct. In the following example, there are no whitespace children of root, but root's next sibling is the
        // new line whitespace.
        //
        //      @JvmField
        //      val s: Any
        //
        let white_spaces: Vec<NodeId> = annotations
            .iter()
            .map(|&it| ast.next_sibling(it))
            .chain(std::iter::once(ast.next_sibling(node)))
            .flatten()
            .filter(|&it| ast.is_white_space(it))
            .take(annotations.len())
            .collect();

        let next = next_sibling_with_at_least_one_of(
            ast,
            node,
            |it| {
                !ast.is_white_space(it)
                    && ast.text_length(it) > 0
                    && !ast.is_part_of(it, FILE_ANNOTATION_LIST)
                    && !is_comment_on_same_line_as_prev_leaf(ast, it)
            },
            |it| {
                // Disallow multiple white spaces as well as comments
                if ast.is_white_space(it) {
                    let s = ast.leaf_text(it);
                    // Ensure at least one occurrence of two line breaks
                    s.find('\n') != s.rfind('\n')
                } else {
                    ast.is_part_of_comment(it) && !is_comment_on_same_line_as_prev_leaf(ast, it)
                }
            },
        );
        if let Some(next) = next
            && ast.element_type(node) != FILE_ANNOTATION_LIST
            && ast.is_part_of_comment(next)
        {
            emit(ast, ast.end_offset(node), ERROR_MESSAGE, true).if_autocorrect_allowed(|| {
                // Special-case autocorrection when the annotation is separated from the annotated construct
                // by a comment: we need to swap the order of the comment and the annotation
                // Remove the annotation and the following whitespace
                let eol_comment = ast.next_sibling_matching(node, |it| is_comment_on_same_line_as_prev_leaf(ast, it));
                if let Some(eol_comment) = eol_comment {
                    if let Some(it) = ast.prev_sibling_matching(eol_comment, |it| ast.is_white_space(it)) {
                        ast.remove(it);
                    }
                    if let Some(it) = ast.next_sibling_matching(eol_comment, |it| ast.is_white_space(it)) {
                        ast.remove(it);
                    }
                    ast.remove(eol_comment);
                } else if let Some(it) = ast.next_sibling_matching(node, |it| ast.is_white_space(it)) {
                    ast.remove(it);
                }
                ast.remove(node);

                // Insert the annotation prior to the annotated construct
                let before_anchor = ast.next_code_sibling(next);
                let parent = ast.parent(next).expect("NullPointerException: parent!!");
                ast.add_child(parent, node, before_anchor);
                if let Some(eol_comment) = eol_comment {
                    let white_space = ast.new_leaf(WHITE_SPACE, " ");
                    ast.add_child(parent, white_space, before_anchor);
                    ast.add_child(parent, eol_comment, before_anchor);
                }
                let white_space = ast.new_leaf(WHITE_SPACE, "\n");
                ast.add_child(parent, white_space, before_anchor);
            });
        }
        if ast.element_type(node) != FILE_ANNOTATION_LIST && white_spaces.iter().any(|&it| contains_multiple_newlines(ast, it)) {
            emit(ast, ast.end_offset(node), ERROR_MESSAGE, true).if_autocorrect_allowed(|| {
                remove_intra_line_breaks(ast, node, *annotations.last().unwrap());
                remove_extra_line_breaks(ast, node);
            });
        }
    }
}

fn contains_multiple_newlines(ast: &Ast, n: NodeId) -> bool {
    ast.text(n).bytes().filter(|&b| b == b'\n').count() > 1
}

fn next_sibling_with_at_least_one_of(
    ast: &Ast,
    this: NodeId,
    p: impl Fn(NodeId) -> bool,
    needs_to_occur: impl Fn(NodeId) -> bool,
) -> Option<NodeId> {
    let mut node = ast.next_sibling(this);
    let mut occurrence_count = 0;
    while let Some(n) = node {
        if needs_to_occur(n) {
            occurrence_count += 1;
        }
        if p(n) {
            return if occurrence_count > 0 { Some(n) } else { None };
        }
        node = ast.next_sibling(n);
    }
    None
}

fn remove_extra_line_breaks(ast: &mut Ast, node: NodeId) {
    let next = ast.next_sibling_matching(node, |it| ast.is_white_space_with_newline(it)).filter(|&it| ast.is_leaf_element(it));
    if let Some(next) = next {
        raw_replace_extra_line_breaks(ast, next);
    }
}

fn raw_replace_extra_line_breaks(ast: &mut Ast, node: NodeId) {
    // Replace the extra white space with a single break
    let text = ast.text(node);
    let newline = text.find('\n');
    let first_index = newline.map_or(0, |i| i + 1);
    let after = newline.map_or(text.as_str(), |i| &text[i + 1..]);
    let replacement_text = format!("{}{}", &text[..first_index], after.replace('\n', ""));

    ast.replace_text_with(node, &replacement_text);
}

fn remove_intra_line_breaks(ast: &mut Ast, from_node: NodeId, last_annotation_entry_node: NodeId) {
    // Pull the next before raw replace, or it will blow up
    let next_leaf = ast.next_leaf(from_node);
    if ast.is_white_space(from_node) && ast.text(from_node).bytes().filter(|&b| b == b'\n').count() > 1 {
        raw_replace_extra_line_breaks(ast, from_node);
    }

    if let Some(next_leaf) = next_leaf
        && !ast.text(last_annotation_entry_node).ends_with(&ast.text(next_leaf))
    {
        remove_intra_line_breaks(ast, next_leaf, last_annotation_entry_node);
    }
}

fn is_comment_on_same_line_as_prev_leaf(ast: &Ast, n: NodeId) -> bool {
    ast.is_part_of_comment(n) && !ast.leaves(n, false).take_while(|&it| ast.is_white_space(it)).any(|it| ast.text_contains(it, '\n'))
}
