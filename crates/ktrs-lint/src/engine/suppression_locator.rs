//! Port of ktlint-rule-engine `internal/SuppressionLocator.kt`: `@Suppress`/`@SuppressWarnings` on the
//! nearest `KtAnnotated` owner and formatter-tag comments, as offset ranges.

use std::collections::BTreeSet;

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::kt_tokens::COMMENTS;
use ktrs_syntax::SyntaxKind::{
    ANNOTATION_ENTRY, CONSTRUCTOR_CALLEE, RBRACE, TYPE_REFERENCE, VALUE_ARGUMENT,
};

use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::EditorConfig;
use crate::engine::ast_helpers::{is_kt_annotated, recursive_children, text_range};
use crate::engine::formatter_tags::FormatterTags;
use crate::rule::RuleV2;

/// `range` is inclusive, in UTF-8 offsets of the tree the hints were built from; empty ids = all rules.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SuppressionHint {
    range: (i64, i64),
    disabled_rule_ids: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct CommentSuppressionHint {
    node: NodeId,
    disabled_rule_ids: BTreeSet<String>,
    block_start: bool,
}

pub struct SuppressionLocator {
    formatter_tags: FormatterTags,
    /// `rootNode.text.hashCode()` of the text the hints were built from, with the cheap fingerprint
    /// that tells when it has to be recomputed.
    hashcode_ast_node_text: Option<i32>,
    fingerprint: Option<(usize, usize)>,
    suppression_hints: Vec<SuppressionHint>,
}

impl SuppressionLocator {
    pub fn new(editor_config: &EditorConfig) -> SuppressionLocator {
        SuppressionLocator {
            formatter_tags: FormatterTags::from(editor_config),
            hashcode_ast_node_text: None,
            fingerprint: None,
            suppression_hints: Vec::new(),
        }
    }

    /// Whether the element at `offset` in `root_node` is suppressed for `rule`. The hints are rebuilt
    /// when the root text changed (format mode mutates it).
    pub fn suppress(
        &mut self,
        ast: &Ast,
        root_node: NodeId,
        offset: usize,
        rule: &dyn RuleV2,
    ) -> bool {
        // Every edit allocates a node or changes the length, so the text is only hashed after one.
        // TODO: use an Ast modification counter (1B) instead of (node_count, text_length).
        let fingerprint = (ast.node_count(), ast.text_length(root_node));
        if self.fingerprint != Some(fingerprint) {
            self.fingerprint = Some(fingerprint);
            let hash_code = java_string_hash_code(&ast.text(root_node));
            if self.hashcode_ast_node_text != Some(hash_code) {
                self.hashcode_ast_node_text = Some(hash_code);
                self.suppression_hints = self.find_suppression_hints(ast, root_node);
            }
        }
        if rule.ignores_ktlint_suppressions() || self.suppression_hints.is_empty() {
            return false;
        }
        let rule_id = rule.rule_id().value();
        self.suppression_hints
            .iter()
            .filter(|it| it.range.0 <= offset as i64 && offset as i64 <= it.range.1)
            .any(|hint| {
                hint.disabled_rule_ids.is_empty() || hint.disabled_rule_ids.contains(rule_id)
            })
    }

    fn find_suppression_hints(&self, ast: &Ast, root_node: NodeId) -> Vec<SuppressionHint> {
        let mut suppression_hints = Vec::new();
        let mut comment_suppressions_hints = Vec::new();
        for node in recursive_children(ast, root_node) {
            let element_type = ast.element_type(node);
            if COMMENTS.contains(element_type) {
                if let Some(hint) = self.create_suppression_hint_from_comment(ast, node) {
                    comment_suppressions_hints.push(hint);
                }
            } else if element_type == ANNOTATION_ENTRY
                && is_suppress_annotation(ast, node)
                && let Some(hint) = create_suppression_hint_from_annotations(ast, node)
            {
                suppression_hints.push(hint);
            }
        }
        suppression_hints.extend(to_suppression_hints(ast, comment_suppressions_hints));
        suppression_hints
    }

    fn create_suppression_hint_from_comment(
        &self,
        ast: &Ast,
        node: NodeId,
    ) -> Option<CommentSuppressionHint> {
        let owned;
        let text = if ast.is_leaf_element(node) {
            ast.leaf_text(node)
        } else {
            owned = ast.text(node);
            &owned
        };
        let text = text.strip_prefix("//").unwrap_or(text);
        let text = text.strip_prefix("/*").unwrap_or(text);
        let text = text.strip_suffix("*/").unwrap_or(text);
        let mut parts = text.trim().split(' ');
        let first = parts.next();
        let tail = || parts.map(str::to_owned).collect();
        if first == self.formatter_tags.formatter_tag_off.as_deref() {
            Some(CommentSuppressionHint {
                node,
                disabled_rule_ids: tail(),
                block_start: true,
            })
        } else if first == self.formatter_tags.formatter_tag_on.as_deref() {
            Some(CommentSuppressionHint {
                node,
                disabled_rule_ids: tail(),
                block_start: false,
            })
        } else {
            None
        }
    }
}

fn to_suppression_hints(
    ast: &Ast,
    comment_suppression_hints: Vec<CommentSuppressionHint>,
) -> Vec<SuppressionHint> {
    let mut suppression_hints = Vec::new();
    let mut block_comment_suppression_hints: Vec<CommentSuppressionHint> = Vec::new();
    for comment_suppression_hint in comment_suppression_hints {
        if comment_suppression_hint.block_start {
            block_comment_suppression_hints.push(comment_suppression_hint);
        } else if let Some(index) = block_comment_suppression_hints
            .iter()
            .rposition(|it| it.disabled_rule_ids == comment_suppression_hint.disabled_rule_ids)
        {
            let open_hint = block_comment_suppression_hints.remove(index);
            suppression_hints.push(SuppressionHint {
                range: (
                    start_offset(ast, open_hint.node),
                    start_offset(ast, comment_suppression_hint.node) - 1,
                ),
                disabled_rule_ids: comment_suppression_hint.disabled_rule_ids,
            });
        }
    }
    // The remaining hints were not closed.
    for it in block_comment_suppression_hints {
        let range = match rbrace_of_containing_block(ast, it.node) {
            // Up to the end of the next sibling when the outer element does not end with a `}`.
            None => {
                let end = ast
                    .next_sibling(it.node)
                    .map_or(text_range(ast, it.node).1, |next| text_range(ast, next).1);
                (start_offset(ast, it.node), end as i64)
            }
            // Up to, but excluding, the `}` of the containing block.
            Some(rbrace) => (start_offset(ast, it.node), start_offset(ast, rbrace) - 1),
        };
        suppression_hints.push(SuppressionHint {
            range,
            disabled_rule_ids: it.disabled_rule_ids,
        });
    }
    suppression_hints
}

fn rbrace_of_containing_block(ast: &Ast, n: NodeId) -> Option<NodeId> {
    ast.parent(n)
        .and_then(|p| ast.last_child_node(p))
        .filter(|&it| ast.element_type(it) == RBRACE)
}

const SUPPRESS_ANNOTATIONS: [&str; 2] = ["Suppress", "SuppressWarnings"];
const ALL_KTLINT_RULES_SUPPRESSION_ID: &str = "ktlint:suppress-all-rules";

fn is_suppress_annotation(ast: &Ast, n: NodeId) -> bool {
    ast.find_child_by_type(n, CONSTRUCTOR_CALLEE)
        .and_then(|it| ast.find_child_by_type(it, TYPE_REFERENCE))
        .is_some_and(|it| SUPPRESS_ANNOTATIONS.contains(&ast.text(it).as_str()))
}

fn create_suppression_hint_from_annotations(ast: &Ast, n: NodeId) -> Option<SuppressionHint> {
    let suppressed_rule_ids: Vec<String> = recursive_children(ast, n)
        .into_iter()
        .filter(|&it| ast.element_type(it) == VALUE_ARGUMENT)
        .flat_map(|it| find_rule_suppression_ids(remove_surrounding(&ast.text(it), "\"")))
        .collect();
    if suppressed_rule_ids.is_empty() {
        return None;
    }
    let owner = ast.parent_matching(n, |it| is_kt_annotated(ast.element_type(it)))?;
    let (start, end) = text_range(ast, owner);
    let disabled_rule_ids = if suppressed_rule_ids
        .iter()
        .any(|id| id == ALL_KTLINT_RULES_SUPPRESSION_ID)
    {
        BTreeSet::new()
    } else {
        suppressed_rule_ids.into_iter().collect()
    };
    Some(SuppressionHint {
        range: (start as i64, end as i64 - 1),
        disabled_rule_ids,
    })
}

fn start_offset(ast: &Ast, n: NodeId) -> i64 {
    ast.start_offset(n) as i64
}

fn find_rule_suppression_ids(value: &str) -> Vec<String> {
    if value == "ktlint" {
        vec![ALL_KTLINT_RULES_SUPPRESSION_ID.to_owned()]
    } else if let Some(rule_id) = value.strip_prefix("ktlint:") {
        vec![rule_id.to_owned()]
    } else {
        suppress_annotation_rule_map(value)
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

/// Non-ktlint suppressions that also suppress the matching ktlint rules.
fn suppress_annotation_rule_map(annotation_value: &str) -> &'static [&'static str] {
    match annotation_value {
        "EnumEntryName" => &["standard:enum-entry-name-case"],
        "RemoveCurlyBracesFromTemplate" => &["standard:string-template"],
        "ClassName" => &["standard:class-naming"],
        "FunctionName" => &["standard:function-naming"],
        "LocalVariableName" => &["standard:backing-property-naming"],
        "PackageName" => &["standard:package-name"],
        "PropertyName" | "ObjectPropertyName" => &[
            "standard:property-naming",
            "standard:backing-property-naming",
        ],
        "ConstPropertyName" | "PrivatePropertyName" => &["standard:property-naming"],
        "UnusedImport" => &["standard:no-unused-imports"],
        _ => &[],
    }
}

/// Kotlin `removeSurrounding(delimiter)`: only when both ends have it and they don't overlap.
pub(crate) fn remove_surrounding<'a>(s: &'a str, delimiter: &str) -> &'a str {
    if s.len() >= 2 * delimiter.len() && s.starts_with(delimiter) && s.ends_with(delimiter) {
        &s[delimiter.len()..s.len() - delimiter.len()]
    } else {
        s
    }
}

/// `String.hashCode()` over UTF-16 code units.
fn java_string_hash_code(text: &str) -> i32 {
    text.encode_utf16()
        .fold(0i32, |h, c| h.wrapping_mul(31).wrapping_add(i32::from(c)))
}
