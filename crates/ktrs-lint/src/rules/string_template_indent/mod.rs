//! Port of ktlint-ruleset-standard `StringTemplateIndentRule.kt`: this file up to `getNonBlankLines`, `indent.rs` from
//! `indentStringTemplate` to the end.

mod indent;

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    BINARY_EXPRESSION, COMMA, DOT_QUALIFIED_EXPRESSION, EQ, FUN, OPERATION_REFERENCE, RETURN_KEYWORD, RPAR, STRING_TEMPLATE,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::engine::kotlin_text::is_kotlin_blank;
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, IndentStyle, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

use indent::{contains_literal_string_template_entry_with_newline, indent_length, is_followed_by_trim_indent, split_indent_at};

const RAW_STRING_LITERAL_QUOTES: &str = "\"\"\"";
const VISITED_TYPES: TokenSet = TokenSet::create(&[STRING_TEMPLATE]);

/// The `lateinit` properties are `None` until `beforeFirstNode` sets them (it skips them when indenting is disabled).
pub struct StringTemplateIndentRule {
    indent_config: IndentConfig,
    next_indent: Option<String>,
    wrong_indent_char: Option<&'static str>,
    wrong_indent_description: Option<&'static str>,
}

impl StringTemplateIndentRule {
    pub fn new() -> StringTemplateIndentRule {
        StringTemplateIndentRule {
            indent_config: IndentConfig::default_indent_config(),
            next_indent: None,
            wrong_indent_char: None,
            wrong_indent_description: None,
        }
    }

    fn next_indent(&self) -> &str {
        self.next_indent.as_deref().expect("UninitializedPropertyAccessException: lateinit property nextIndent has not been initialized")
    }

    fn wrong_indent_char(&self) -> &'static str {
        self.wrong_indent_char
            .expect("UninitializedPropertyAccessException: lateinit property wrongIndentChar has not been initialized")
    }

    fn wrong_indent_description(&self) -> &'static str {
        self.wrong_indent_description
            .expect("UninitializedPropertyAccessException: lateinit property wrongIndentDescription has not been initialized")
    }
}

impl Default for StringTemplateIndentRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for StringTemplateIndentRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:string-template-indent")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*INDENT_SIZE_PROPERTY), PropertyRef::from(&*INDENT_STYLE_PROPERTY)]
    }

    fn is_official_code_style(&self) -> bool {
        true
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        if self.indent_config.disabled() {
            return;
        }

        self.next_indent = Some(self.indent_config.indent.clone());
        match self.indent_config.indent_style {
            IndentStyle::Space => {
                self.wrong_indent_char = Some("\t");
                self.wrong_indent_description = Some("tab");
            }
            IndentStyle::Tab => {
                self.wrong_indent_char = Some(" ");
                self.wrong_indent_description = Some("space");
            }
        }
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != STRING_TEMPLATE {
            return;
        }
        let string_template = node;
        if contains_literal_string_template_entry_with_newline(ast, string_template) && is_followed_by_trim_indent(ast, string_template) {
            if !is_preceded_by_whitespace_with_newline(ast, string_template)
                && !is_preceded_by_return_keyword(ast, string_template)
                && !is_function_body_expression_on_same_line(ast, string_template)
            {
                let white_space = string_template;
                emit(ast, ast.start_offset(string_template), "Expected newline before multiline string template", true)
                    .if_autocorrect_allowed(|| {
                        let indent = self.indent_config.child_indent_of(ast, ast.parent(white_space).expect("NullPointerException: parent!!"));
                        ast.upsert_whitespace_before_me(white_space, &indent);
                    });
            }
            if let Some(next_leaf) = get_first_leaf_after_trim_indent(ast, string_template)
                .filter(|&it| !ast.is_white_space_with_newline(it))
                .filter(|&it| ast.element_type(it) != COMMA)
                .filter(|&it| ast.parent(it).map(|p| ast.element_type(p)) != Some(DOT_QUALIFIED_EXPRESSION))
                .filter(|&it| {
                    !(ast.parent(it).map(|p| ast.element_type(p)) == Some(BINARY_EXPRESSION)
                        && ast.next_sibling(it).map(|s| ast.element_type(s)) == Some(OPERATION_REFERENCE))
                })
            {
                emit(ast, ast.start_offset(next_leaf), "Expected newline after multiline string template", true).if_autocorrect_allowed(
                    || {
                        let parent = ast.parent(string_template).expect("NullPointerException: parent!!");
                        let indent = self.indent_config.child_indent_of(ast, parent);
                        ast.upsert_whitespace_before_me(next_leaf, &indent);
                    },
                );
            }

            if contains_mixed_indentation_characters(ast, string_template) {
                // It can not be determined with certainty how mixed indentation characters should be interpreted. The trimIndent
                // function handles tabs and spaces equally (one tabs equals one space) while the user might expect that the tab size in
                // the indentation is more than one space.
                emit(
                    ast,
                    ast.start_offset(string_template),
                    "Indentation of multiline raw string literal should not contain both tab(s) and space(s)",
                    false,
                );
                return;
            }

            let indent = self.get_indent(ast, string_template);
            self.indent_string_template(ast, node, &indent, emit);
        }
    }
}

fn get_first_leaf_after_trim_indent(ast: &Ast, this: NodeId) -> Option<NodeId> {
    Some(this)
        .filter(|&it| ast.element_type(it) == STRING_TEMPLATE)
        .filter(|&it| is_followed_by_trim_indent(ast, it))
        .and_then(|it| ast.parent(it))
        .map(|it| ast.last_child_leaf_or_self(it))
        .and_then(|it| ast.next_leaf(it))
}

fn is_preceded_by_whitespace_with_newline(ast: &Ast, this: NodeId) -> bool {
    ast.is_white_space_with_newline(ast.prev_leaf(this))
}

// Allow below as otherwise it results in compilation failure:
//   return """
//       some string
//       """
fn is_preceded_by_return_keyword(ast: &Ast, this: NodeId) -> bool {
    ast.prev_code_leaf(this).map(|it| ast.element_type(it)) == Some(RETURN_KEYWORD)
}

fn is_function_body_expression_on_same_line(ast: &Ast, this: NodeId) -> bool {
    ast.prev_code_leaf(this)
        .filter(|&it| ast.element_type(it) == EQ)
        .and_then(|it| closing_parenthesis_of_function_or_null(ast, it))
        .and_then(|it| ast.prev_leaf(it))
        .is_some_and(|it| ast.is_white_space_with_newline(it))
}

fn closing_parenthesis_of_function_or_null(ast: &Ast, this: NodeId) -> Option<NodeId> {
    Some(this)
        .filter(|&it| ast.parent(it).map(|p| ast.element_type(p)) == Some(FUN))
        .and_then(|it| ast.prev_code_leaf(it))
        .filter(|&it| ast.element_type(it) == RPAR)
}

impl StringTemplateIndentRule {
    // When executing this rule, the indentation rule may not have run yet. The functionality to determine the correct indentation level
    // is out of scope of this rule as it is owned by the indentation rule. Therefore, the indentation of the line at which the
    // string template is found, is assumed to be correct and is used to indent all lines of the string template. The indent will be
    // fixed once the indent rule is run as well.
    fn get_indent(&self, ast: &Ast, this: NodeId) -> String {
        let first_white_space_leaf_on_same_line = ast.prev_leaf_matching(this, |it| ast.is_white_space_with_newline(it));
        if ast.prev_leaf(this) == first_white_space_leaf_on_same_line {
            // The string template already is on a separate new line. Keep the current indent.
            get_text_after_last_newline(ast, first_white_space_leaf_on_same_line)
        } else {
            // String template is forced to a new line. So indent must be increased
            get_text_after_last_newline(ast, first_white_space_leaf_on_same_line) + self.next_indent()
        }
    }
}

fn get_text_after_last_newline(ast: &Ast, this: Option<NodeId>) -> String {
    this.map(|it| {
        let text = ast.text(it);
        text.rsplit('\n').next().unwrap_or("").to_owned()
    })
    .unwrap_or_default()
}

fn contains_mixed_indentation_characters(ast: &Ast, this: NodeId) -> bool {
    assert!(contains_literal_string_template_entry_with_newline(ast, this), "IllegalArgumentException: Failed requirement.");
    let non_blank_lines = get_non_blank_lines(ast, this);
    let prefix_length = non_blank_lines.iter().map(|it| indent_length(it)).min().unwrap_or(0);
    let mut distinct_indent_characters: Vec<char> = Vec::new();
    for line in &non_blank_lines {
        for c in split_indent_at(line, prefix_length).0.chars() {
            if !distinct_indent_characters.contains(&c) {
                distinct_indent_characters.push(c);
            }
        }
    }
    distinct_indent_characters.len() > 1
}

fn get_non_blank_lines(ast: &Ast, this: NodeId) -> Vec<String> {
    assert!(ast.element_type(this) == STRING_TEMPLATE, "IllegalArgumentException: Failed requirement.");
    ast.text(this)
        .split('\n')
        .map(|it| it.strip_prefix(RAW_STRING_LITERAL_QUOTES).unwrap_or(it))
        .map(|it| it.strip_suffix(RAW_STRING_LITERAL_QUOTES).unwrap_or(it))
        .filter(|it| !is_kotlin_blank(it))
        .map(str::to_owned)
        .collect()
}
