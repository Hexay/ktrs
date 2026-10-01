//! Port of ktlint-ruleset-standard `BlockCommentInitialStarAlignmentRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::BLOCK_COMMENT;

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeLines;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[BLOCK_COMMENT]);

/// When present, align the initial star in a block comment.
pub struct BlockCommentInitialStarAlignmentRule;

impl RuleV2 for BlockCommentInitialStarAlignmentRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:block-comment-initial-star-alignment")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == BLOCK_COMMENT {
            let expected_indent_for_line_with_initial_star = ast.indent_without_newline_prefix(node) + " *";
            let text = ast.leaf_text(node).to_owned();
            let mut offset = ast.start_offset(node);
            let mut modified_lines: Vec<String> = Vec::new();
            let mut autocorrect = false;
            for line in text.split('\n') {
                let modified_line = match continuation_comment_regex_find(line) {
                    Some((prefix, content)) => {
                        if prefix != expected_indent_for_line_with_initial_star {
                            emit(ast, offset + prefix.len(), "Initial star should align with start of block comment", true)
                                .if_autocorrect_allowed(|| autocorrect = true);
                            expected_indent_for_line_with_initial_star.clone() + content
                        } else {
                            line.to_owned()
                        }
                    }
                    None => line.to_owned(),
                };
                modified_lines.push(modified_line);
                offset += line.len() + 1;
            }
            if autocorrect {
                let new_text = modified_lines.join("\n");
                if text != new_text {
                    ast.replace_text_with(node, &new_text);
                }
            }
        }
    }
}

/// `CONTINUATION_COMMENT_REGEX.find(line)?.destructured` for `^([\t ]+\*)(.*)$` (Java `find`, no flags): the prefix
/// and content groups. `.` stops at a line terminator, and `$` also matches before a final one.
fn continuation_comment_regex_find(line: &str) -> Option<(&str, &str)> {
    let star = line.find(|c: char| c != '\t' && c != ' ')?;
    if star == 0 || !line[star..].starts_with('*') {
        return None;
    }
    let (prefix, rest) = line.split_at(star + 1);
    let is_line_terminator = |c: char| matches!(c, '\n' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}');
    match rest.find(is_line_terminator) {
        None => Some((prefix, rest)),
        Some(i) => {
            let tail = &rest[i..];
            let final_terminator = tail == "\r\n" || tail.chars().count() == 1;
            final_terminator.then(|| (prefix, &rest[..i]))
        }
    }
}
