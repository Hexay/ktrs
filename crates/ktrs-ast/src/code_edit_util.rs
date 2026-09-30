//! Port of IntelliJ `CodeEditUtil`'s removal path (`removeChild` -> `removeChildren` ->
//! `makePlaceHolderBetweenTokens`), which every `PsiElement.delete()` takes (`CompositeElement.deleteChildInternal`).
//! It is the one IntelliJ edit that merges, drops, re-creates or inserts whitespace around the removed range.
//! Free functions, like the Java statics: `CodeEditUtil.removeChild(parent, child)` ->
//! `code_edit_util::remove_child(ast, parent, child)`.
//!
//! The formatter marks (`markToReformatBefore`, `saveWhitespacesInfo`: copyable user data) are not ported:
//! nothing in ktlint reads them. The whitespace a `TokenSeparatorGenerator` inserts is ported: IntelliJ's
//! default generator asks `KotlinCommonParserDefinition.spaceExistanceTypeBetweenTokens`.
//! Checked dump for dump against the ktlint jar: `tests/edit_oracle.rs`.

use ktrs_parser::kt_tokens::is_keyword_token;
use ktrs_syntax::SyntaxKind::{self, *};

use crate::arena::{Ast, NodeId};
use crate::tree_util::{self, CommonParentState};

pub fn remove_child(ast: &mut Ast, parent: NodeId, child: NodeId) {
    remove_children(ast, parent, child, child);
}

/// `removeChildren(parent, first, last)`: `first..=last` must be consecutive children of `parent`.
pub fn remove_children(ast: &mut Ast, parent: NodeId, first: NodeId, last: NodeId) {
    let tailing_element =
        ast.start_offset(last) + ast.text_length(last) == ast.start_offset(parent) + ast.text_length(parent);
    let force_reformat = need_to_force_reformat(ast, Some(parent), first, last);
    let mut child = Some(first);
    while let Some(c) = child.filter(|&c| c != last) {
        child = ast.tree_next(c);
    }
    assert!(child == Some(last), "AssertionError: last is not a successor of first in the .getTreeNext() chain");
    let prev_leaf = tree_util::prev_leaf(ast, first);
    let next_leaf = tree_util::next_leaf(ast, last);
    let first_which_stay_in_tree = ast.tree_next(last);
    ast.remove_range(parent, first, first_which_stay_in_tree);
    let mut next_leaf_to_adjust = next_leaf;
    if let (Some(next), Some(prev)) = (next_leaf_to_adjust, prev_leaf)
        && ast.tree_parent(next).is_none()
    {
        next_leaf_to_adjust = ast.tree_next(prev);
    }
    make_place_holder_between_tokens(ast, prev_leaf, next_leaf_to_adjust, force_reformat, tailing_element);
}

/// True unless `first` starts `parent` without being, trimmed, all of it (recursively up to the root):
/// so true for nearly every removal, which makes the whitespace branches below merge or re-create.
fn need_to_force_reformat(ast: &Ast, parent: Option<NodeId>, first: NodeId, last: NodeId) -> bool {
    let Some(parent) = parent else { return true };
    ast.start_offset(first) != ast.start_offset(parent)
        || java_trim(&ast.text(parent)).len() == get_trimmed_text_length(ast, first, last)
            && need_to_force_reformat(ast, ast.tree_parent(parent), parent, parent)
}

fn get_trimmed_text_length(ast: &Ast, first: NodeId, last: NodeId) -> usize {
    let mut buffer = String::new();
    let end = ast.tree_next(last);
    let mut first = Some(first);
    while first != end {
        let f = first.expect("NullPointerException: getTrimmedTextLength past the last child");
        buffer.push_str(&ast.text(f));
        first = ast.tree_next(f);
    }
    java_trim(&buffer).len()
}

/// Returns the node left standing on the left (`left`, a merged whitespace, or `right`).
fn make_place_holder_between_tokens(
    ast: &mut Ast,
    left: Option<NodeId>,
    right: Option<NodeId>,
    force_reformat: bool,
    normalize_tailing_whitespace: bool,
) -> Option<NodeId> {
    let Some(right) = right else { return left };
    let mut left = left?;
    if ast.element_type(left) == WHITE_SPACE && ast.tree_next(left).is_none() && normalize_tailing_whitespace {
        let prev_leaf = tree_util::prev_leaf(ast, left);
        let parent = tree_parent_or_npe(ast, left);
        ast.remove_child(parent, left);
        mark_to_reformat_before_or_insert_whitespace(ast, prev_leaf, right);
        left = right;
    } else if ast.element_type(left) == WHITE_SPACE && ast.element_type(right) == WHITE_SPACE {
        let (left_text, right_text) = (ast.text(left), ast.text(right));
        let left_blank_lines = get_blank_lines(&left_text);
        let right_blank_lines = get_blank_lines(&right_text);
        let leave_right_text = left_blank_lines < right_blank_lines;
        let text = if left_blank_lines == 0 && right_blank_lines == 0 {
            left_text + &right_text
        } else if leave_right_text {
            right_text
        } else {
            left_text
        };
        if leave_right_text || force_reformat {
            let merged = ast.new_leaf(WHITE_SPACE, &text);
            if !leave_right_text {
                let parent = tree_parent_or_npe(ast, left);
                ast.replace_child(parent, left, merged);
                let parent = tree_parent_or_npe(ast, right);
                ast.remove_child(parent, right);
            } else {
                let parent = tree_parent_or_npe(ast, right);
                ast.replace_child(parent, right, merged);
                let parent = tree_parent_or_npe(ast, left);
                ast.remove_child(parent, left);
            }
            left = merged;
        } else {
            let parent = tree_parent_or_npe(ast, right);
            ast.remove_child(parent, right);
        }
    } else if ast.element_type(left) != WHITE_SPACE || force_reformat {
        if ast.element_type(right) == WHITE_SPACE {
            mark_whitespace_for_reformat(ast, right);
        } else if ast.element_type(left) == WHITE_SPACE {
            mark_whitespace_for_reformat(ast, left);
        } else {
            mark_to_reformat_before_or_insert_whitespace(ast, Some(left), right);
        }
    }
    Some(left)
}

/// Swaps `whitespace` for a fresh whitespace leaf with the same text: a new node identity.
fn mark_whitespace_for_reformat(ast: &mut Ast, whitespace: NodeId) {
    let text = ast.text(whitespace);
    let new_whitespace = ast.new_leaf(WHITE_SPACE, &text);
    let parent = tree_parent_or_npe(ast, whitespace);
    ast.replace_child(parent, whitespace, new_whitespace);
}

fn mark_to_reformat_before_or_insert_whitespace(ast: &mut Ast, left: Option<NodeId>, right: NodeId) {
    let Some(left) = left.filter(|&l| has_language(ast, l) && has_language(ast, right)) else { return };
    let Some(generated) = generate_whitespace_between_tokens(ast, left, right) else { return };
    let mut state = CommonParentState::default();
    tree_util::prev_leaf_with_state(ast, right, &mut state);
    let next_leaf_branch_start = state.next_leaf_branch_start.expect("prevLeaf sets nextLeafBranchStart");
    let parent = tree_parent_or_npe(ast, next_leaf_branch_start);
    ast.add_child(parent, generated, Some(next_leaf_branch_start));
}

fn get_blank_lines(text: &str) -> usize {
    text.matches('\n').count()
}

/// `LanguageTokenSeparatorGenerators`' default generator: `" "` for `MUST`, `"\n"` for `MUST_LINE_BREAK`.
fn generate_whitespace_between_tokens(ast: &mut Ast, left: NodeId, right: NodeId) -> Option<NodeId> {
    let text = match space_existance_type_between_tokens(ast, left, right) {
        SpaceRequirements::Must => " ",
        SpaceRequirements::MustLineBreak => "\n",
        SpaceRequirements::May => return None,
    };
    Some(ast.new_leaf(WHITE_SPACE, text))
}

enum SpaceRequirements {
    May,
    Must,
    MustLineBreak,
}

/// `KotlinCommonParserDefinition.spaceExistanceTypeBetweenTokens(left, right)`.
fn space_existance_type_between_tokens(ast: &Ast, left: NodeId, right: NodeId) -> SpaceRequirements {
    let right_type = ast.element_type(right);
    if matches!(right_type, GET_KEYWORD | SET_KEYWORD) {
        return SpaceRequirements::MustLineBreak;
    }
    let left_type = ast.element_type(left);
    if is_keyword_token(left_type) && is_keyword_token(right_type) {
        return SpaceRequirements::Must;
    }
    if let Some(right_when_entry) = get_parent_of_type(ast, right, WHEN_ENTRY)
        && let Some(left_when_entry) = get_parent_of_type(ast, left, WHEN_ENTRY)
        && left_when_entry != right_when_entry
        && left_type != SEMICOLON
    {
        return SpaceRequirements::MustLineBreak;
    }
    SpaceRequirements::May
}

/// `PsiTreeUtil.getParentOfType(element, KtX, strict = false)` for a class with exactly one element type;
/// the walk stops at a `PsiFile` (the file root or a dummy holder).
fn get_parent_of_type(ast: &Ast, element: NodeId, kind: SyntaxKind) -> Option<NodeId> {
    let mut element = Some(element);
    while let Some(e) = element {
        if ast.element_type(e) == kind && !ast.is_leaf_element(e) {
            return Some(e);
        }
        if ast.is_file_element(e) {
            return None;
        }
        element = ast.tree_parent(e);
    }
    None
}

/// `PsiUtilCore.getNotAnyLanguage(node) != Language.ANY`: Kotlin and KDoc types have a language (and one
/// is a dialect of the other); `TokenType` ones (whitespace, error, dummy holder) defer to the parent.
fn has_language(ast: &Ast, node: NodeId) -> bool {
    let mut node = Some(node);
    while let Some(n) = node {
        if !matches!(ast.element_type(n), WHITE_SPACE | ERROR_ELEMENT | DUMMY_HOLDER | BAD_CHARACTER) {
            return true;
        }
        node = ast.tree_parent(n);
    }
    false
}

fn tree_parent_or_npe(ast: &Ast, node: NodeId) -> NodeId {
    ast.tree_parent(node).expect("NullPointerException: getTreeParent() of a detached node")
}

/// Java's `String.trim()`: strips chars up to U+0020.
fn java_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c <= ' ')
}
