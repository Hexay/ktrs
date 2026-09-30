//! Ports of ktlint-rule-engine `api/KtlintRuleEngineSuppression.kt` (`insertSuppression`) and
//! `api/EditorConfigPropertyRegistry.kt`.

use std::collections::BTreeSet;

use ktrs_ast::{Ast, NodeId};

use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{
    CODE_STYLE_PROPERTY, END_OF_LINE_PROPERTY, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY,
    INSERT_FINAL_NEWLINE_PROPERTY, MAX_LINE_LENGTH_PROPERTY, PropertyRef, RuleExecution,
    create_rule_execution_editor_config_property, create_rule_set_execution_editor_config_property,
};
use crate::engine::code::{Code, KtLintException};
use crate::engine::ktlint_rule_engine::KtLintRuleEngine;
use crate::engine::ktlint_suppression::insert_ktlint_rule_suppression;
use crate::engine::rule_execution_context::create_rule_execution_context;
use crate::rule::{RuleId, RuleSetId};
use crate::rule_provider::RuleV2Provider;

/// `KtlintSuppression`: for the whole file, or at a (1-based) line and column of the code.
#[derive(Clone, Debug)]
pub enum KtlintSuppression {
    ForFile {
        rule_id: RuleId,
    },
    AtOffset {
        line: usize,
        col: usize,
        rule_id: RuleId,
    },
}

impl KtlintSuppression {
    fn rule_id(&self) -> RuleId {
        match self {
            KtlintSuppression::ForFile { rule_id }
            | KtlintSuppression::AtOffset { rule_id, .. } => *rule_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KtlintSuppressionException {
    /// "Offset (line,col) is invalid".
    OutOfBounds {
        line: usize,
        col: usize,
    },
    /// "No ASTNode found at offset (line,col)".
    NoElementFound {
        line: usize,
        col: usize,
    },
    Engine(KtLintException),
}

impl KtLintRuleEngine {
    /// Inserts a `@Suppress` for `suppression` where one may go and returns the code (not otherwise
    /// formatted: other violations are left as they are).
    pub fn insert_suppression(
        &self,
        code: &Code,
        suppression: &KtlintSuppression,
    ) -> Result<String, KtlintSuppressionException> {
        let mut ast = create_rule_execution_context(self, code)
            .map_err(KtlintSuppressionException::Engine)?
            .ast;
        let root_node = ast.root();
        let node = find_leaf_element_at(&ast, root_node, suppression)?;
        insert_ktlint_rule_suppression(
            &mut ast,
            node,
            &BTreeSet::from([suppression.rule_id().value().to_owned()]),
            false,
        );
        Ok(ast.text(root_node))
    }
}

fn find_leaf_element_at(
    ast: &Ast,
    root: NodeId,
    suppression: &KtlintSuppression,
) -> Result<NodeId, KtlintSuppressionException> {
    let KtlintSuppression::AtOffset { line, col, .. } = *suppression else {
        return Ok(root);
    };
    let text = ast.text(root);
    let utf16_offset = offset_from_start_of(&text, line, col)?;
    let leaf = byte_offset_of_utf16(&text, utf16_offset)
        .and_then(|offset| leaf_element_at(ast, root, offset));
    match leaf {
        // A suppression can't go on whitespace: its parent gets it.
        Some(it) if ast.is_white_space(it) => ast
            .parent(it)
            .ok_or(KtlintSuppressionException::NoElementFound { line, col }),
        Some(it) => Ok(it),
        None => Err(KtlintSuppressionException::NoElementFound { line, col }),
    }
}

/// `offsetFromStartOf(code)` in UTF-16 units. At the end of a line (`col == length + 1`) it skips the
/// stripped newline and lands on the next line, as upstream does.
fn offset_from_start_of(
    code: &str,
    line: usize,
    col: usize,
) -> Result<usize, KtlintSuppressionException> {
    let out_of_bounds = KtlintSuppressionException::OutOfBounds { line, col };
    if line < 1 || col < 1 {
        return Err(out_of_bounds);
    }
    let lines: Vec<&str> = code.split('\n').collect();
    if line > lines.len() {
        return Err(out_of_bounds);
    }
    let start_offset_of_line_containing_lint_error: usize = lines[..line - 1]
        .iter()
        .map(|text| utf16_len(text) + 1)
        .sum();
    let code_line_length = utf16_len(lines[line - 1]);
    if col == 1 && code_line_length == 0 {
        Ok(start_offset_of_line_containing_lint_error)
    } else if col <= code_line_length {
        Ok(start_offset_of_line_containing_lint_error + (col - 1))
    } else if col == code_line_length + 1 {
        Ok(start_offset_of_line_containing_lint_error + col)
    } else {
        Err(out_of_bounds)
    }
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

fn byte_offset_of_utf16(text: &str, utf16_offset: usize) -> Option<usize> {
    let mut units = 0;
    for (i, c) in text.char_indices() {
        if units >= utf16_offset {
            return Some(i);
        }
        units += c.len_utf16();
    }
    (units >= utf16_offset).then_some(text.len())
}

/// `ASTNode.findLeafElementAt(offset)`: the leaf containing `offset` (none at the very end).
fn leaf_element_at(ast: &Ast, n: NodeId, offset: usize) -> Option<NodeId> {
    if ast.is_leaf_element(n) {
        return Some(n);
    }
    let mut offset = offset;
    for child in ast.children(n) {
        let length = ast.text_length(child);
        if offset < length {
            return leaf_element_at(ast, child, offset);
        }
        offset -= length;
    }
    None
}

/// `EditorConfigPropertyRegistry`: the `EditorConfigProperty` for a name as it appears in `.editorconfig`,
/// among ktlint's core properties and those of the given rules, or a rule (set) execution property.
pub struct EditorConfigPropertyRegistry {
    properties: Vec<PropertyRef>,
}

impl EditorConfigPropertyRegistry {
    pub fn new(rule_v2_providers: &[RuleV2Provider]) -> EditorConfigPropertyRegistry {
        let mut properties: Vec<PropertyRef> = Vec::new();
        let core = [
            PropertyRef::from(&*CODE_STYLE_PROPERTY),
            PropertyRef::from(&*END_OF_LINE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INSERT_FINAL_NEWLINE_PROPERTY),
            PropertyRef::from(&*MAX_LINE_LENGTH_PROPERTY),
        ];
        for p in rule_v2_providers
            .iter()
            .flat_map(|p| p.uses_editor_config_properties().iter().cloned())
            .chain(core)
        {
            if !properties.iter().any(|e| e.identity() == p.identity()) {
                properties.push(p);
            }
        }
        EditorConfigPropertyRegistry { properties }
    }

    /// `find(propertyName)`; `Err` is the `EditorConfigPropertyNotFoundException` message.
    pub fn find(&self, property_name: &str) -> Result<PropertyRef, String> {
        if let Some(p) = self
            .properties
            .iter()
            .find(|p| p.property_type().name() == property_name)
        {
            return Ok(p.clone());
        }
        if let Some(p) = to_rule_execution_property_or_null(property_name) {
            return Ok(p);
        }
        let mut names: Vec<&str> = self
            .properties
            .iter()
            .map(|p| p.property_type().name())
            .collect();
        names.sort_unstable();
        Err(format!(
            "Property with name '{property_name}' is not found in any of given rules. Available properties:\n\t{}\nNext to properties \
             above, the properties to enable or disable ktlint rules are allowed as well.",
            names
                .iter()
                .map(|n| format!("- {n}"))
                .collect::<Vec<_>>()
                .join("\n\t")
        ))
    }
}

/// `ktlint_<set>_<rule>` / `ktlint_<set>`: every `_` becomes `:`, so only ids without `_` resolve.
fn to_rule_execution_property_or_null(property_name: &str) -> Option<PropertyRef> {
    let id = property_name.strip_prefix("ktlint_")?.replace('_', ":");
    if RuleId::is_valid(&id) {
        Some(create_rule_execution_editor_config_property(&id, RuleExecution::Enabled).into())
    } else if RuleSetId::is_valid(&id) {
        Some(create_rule_set_execution_editor_config_property(&id, RuleExecution::Enabled).into())
    } else {
        None
    }
}
