//! Port of ktlint-ruleset-standard `MaxLineLengthRule.kt`, with `EditorConfig.maxLineLength()`, which other rules call.

use std::sync::LazyLock;

use ktrs_ast::{Ast, NodeId};
use ktrs_editorconfig::PropertyType;
use ktrs_editorconfig::property_type::boolean_value_parser;

use crate::ast_node_extension::{AstNodeExtension, AstNodeLines};
use crate::element_type::{COMMA, IDENTIFIER, IMPORT_DIRECTIVE, KDOC, PACKAGE_DIRECTIVE, STRING_TEMPLATE};
use crate::editorconfig::{
    EditorConfig, EditorConfigProperty, MAX_LINE_LENGTH_PROPERTY, MAX_LINE_LENGTH_PROPERTY_OFF, PropertyRef,
    RULE_EXECUTION_PROPERTY_TYPE, RuleExecution, rule_execution_property_name,
};
use crate::rule::{About, Emit, RuleId, RuleV2, TraversalState};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct MaxLineLengthRule {
    max_line_length: i32,
    ignore_back_ticked_identifier: bool,
    traversal_state: TraversalState,
}

impl MaxLineLengthRule {
    pub fn new() -> MaxLineLengthRule {
        MaxLineLengthRule {
            max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value,
            ignore_back_ticked_identifier: IGNORE_BACKTICKED_IDENTIFIER_PROPERTY.default_value,
            traversal_state: TraversalState::default(),
        }
    }
}

impl Default for MaxLineLengthRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for MaxLineLengthRule {
    fn rule_id(&self) -> RuleId {
        MAX_LINE_LENGTH_RULE_ID
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*MAX_LINE_LENGTH_PROPERTY), PropertyRef::from(&*IGNORE_BACKTICKED_IDENTIFIER_PROPERTY)]
    }

    fn traversal_state(&self) -> Option<TraversalState> {
        Some(self.traversal_state)
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.ignore_back_ticked_identifier = editor_config.get(&IGNORE_BACKTICKED_IDENTIFIER_PROPERTY);
        self.max_line_length = max_line_length(editor_config);
        if self.max_line_length == MAX_LINE_LENGTH_PROPERTY_OFF {
            self.traversal_state.stop_traversal_of_ast();
        }
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.is_white_space(node) {
            return;
        }
        let max_line_length = self.max_line_length as usize;
        let matches = Some(node)
            .filter(|&it| ast.is_leaf_element(it))
            .filter(|&it| ast.next_leaf(it).is_none_or(|n| ast.is_white_space_with_newline(n)))
            .filter(|&it| self.line_length(ast, it) > max_line_length)
            .filter(|&it| !ast.is_part_of(it, PACKAGE_DIRECTIVE))
            .filter(|&it| !ast.is_part_of(it, IMPORT_DIRECTIVE))
            .filter(|&it| !ast.is_part_of(it, KDOC))
            .filter(|&it| !is_part_of_raw_multi_line_string(ast, it))
            .filter(|&it| !is_line_only_containing_single_template_string(ast, it))
            .filter(|&it| {
                !(ast.element_type(it) == COMMA
                    && ast.prev_leaf(it).is_some_and(|p| is_line_only_containing_single_template_string(ast, p)))
            })
            .filter(|&it| !is_line_only_containing_comment(ast, it));
        if matches.is_some() {
            // Calculate the offset at the last possible position at which the newline should be inserted on the line
            let first = ast.leaves_on_line(node).next().expect("NoSuchElementException: Sequence is empty.");
            let offset = utf16_units_forward(ast, first, max_line_length + 1);
            emit(ast, offset, &format!("Exceeded max line length ({max_line_length})"), false);
        }
    }
}

impl MaxLineLengthRule {
    fn line_length(&self, ast: &Ast, node: NodeId) -> usize {
        ast.line_length(ast.leaves_on_line(node).filter(|&it| {
            !(self.ignore_back_ticked_identifier
                && ast.element_type(it) == IDENTIFIER
                && is_back_ticked_identifier(ast.leaf_text(it)))
        }))
    }
}

/// `BACKTICKED_IDENTIFIER_REGEX` = `` `.*` `` (whole text).
fn is_back_ticked_identifier(text: &str) -> bool {
    text.len() >= 2 && text.starts_with('`') && text.ends_with('`') && !text.contains(['\n', '\r'])
}

fn is_part_of_raw_multi_line_string(ast: &Ast, node: NodeId) -> bool {
    ast.find_parent_by_type(node, STRING_TEMPLATE).is_some_and(|it| {
        ast.first_child_node(it).is_some_and(|first| ast.text_matches(first, "\"\"\"")) && ast.text_contains(it, '\n')
    })
}

fn is_line_only_containing_single_template_string(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .filter(|&it| ast.element_type(it) == STRING_TEMPLATE)
        .is_some_and(|string_template| ast.prev_leaf(string_template).is_none_or(|it| ast.is_white_space_with_newline(it)))
}

fn is_line_only_containing_comment(ast: &Ast, node: NodeId) -> bool {
    ast.is_part_of_comment(node) && ast.prev_leaf(node).is_none_or(|it| ast.is_white_space_with_newline(it))
}

/// `startOffset + units` of `from`, counted in UTF-16 units (as the JVM does), as a UTF-8 offset for `emit`.
fn utf16_units_forward(ast: &Ast, from: NodeId, units: usize) -> usize {
    let start = ast.start_offset(from);
    if ast.is_ascii() {
        return start + units;
    }
    let (mut bytes, mut remaining) = (0, units);
    for leaf in ast.leaves_forwards_including_self(from) {
        for c in ast.text(leaf).chars() {
            if remaining < c.len_utf16() {
                return start + bytes;
            }
            remaining -= c.len_utf16();
            bytes += c.len_utf8();
        }
    }
    start + bytes + remaining
}

pub static IGNORE_BACKTICKED_IDENTIFIER_PROPERTY_TYPE: PropertyType<bool> = PropertyType {
    name: "ktlint_ignore_back_ticked_identifier",
    description: "Defines whether the backticked identifier (``) should be ignored",
    parser: boolean_value_parser,
    possible_values: &["true", "false"],
    lower_casing: true,
};

pub static IGNORE_BACKTICKED_IDENTIFIER_PROPERTY: LazyLock<EditorConfigProperty<bool>> =
    LazyLock::new(|| EditorConfigProperty::new(&IGNORE_BACKTICKED_IDENTIFIER_PROPERTY_TYPE, false));

/// Gets the max_line_length property if the `max-line-length` rule is enabled. Otherwise, returns `Int.MAX_VALUE`.
pub fn max_line_length(editor_config: &EditorConfig) -> i32 {
    if max_line_length_rule_enabled(editor_config) { editor_config.get(&MAX_LINE_LENGTH_PROPERTY) } else { i32::MAX }
}

fn max_line_length_rule_enabled(editor_config: &EditorConfig) -> bool {
    Some(RuleExecution::Enabled)
        == editor_config
            .get_editor_config_value_or_null(&RULE_EXECUTION_PROPERTY_TYPE, &rule_execution_property_name(MAX_LINE_LENGTH_RULE_ID.value()))
}

pub const MAX_LINE_LENGTH_RULE_ID: RuleId = RuleId("standard:max-line-length");
