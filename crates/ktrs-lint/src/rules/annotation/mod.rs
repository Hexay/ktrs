//! Port of ktlint-ruleset-standard `AnnotationRule.kt`, split in upstream order: this file (the rule up to
//! `wrapTypeArgumentList`) and `entries.rs` (`visitAnnotationEntry` to the end).

mod entries;

use std::sync::LazyLock;

use ktrs_ast::{Ast, NodeId};
use ktrs_editorconfig::PropertyType;
use ktrs_syntax::SyntaxKind::{
    self, ANNOTATED_EXPRESSION, ANNOTATION, ANNOTATION_ENTRY, CONSTRUCTOR_KEYWORD, FILE_ANNOTATION_LIST, GT, LAMBDA_EXPRESSION,
    MODIFIER_LIST, OPERATION_REFERENCE, TYPE_ARGUMENT_LIST, TYPE_PROJECTION, TYPE_REFERENCE, VALUE_ARGUMENT, VALUE_PARAMETER,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{
    CODE_STYLE_PROPERTY, CodeStyleValue, EditorConfigProperty, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, KtlintVersion, PropertyRef,
    comma_separated_list_value_parser,
};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TokenSet, VisitorModifier};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[
    FILE_ANNOTATION_LIST,
    ANNOTATED_EXPRESSION,
    MODIFIER_LIST,
    ANNOTATION,
    ANNOTATION_ENTRY,
    TYPE_ARGUMENT_LIST,
]);
const ANNOTATION_CONTAINER: [SyntaxKind; 3] = [ANNOTATED_EXPRESSION, FILE_ANNOTATION_LIST, MODIFIER_LIST];
const FAILED_REQUIREMENT: &str = "IllegalArgumentException: Failed requirement.";

static ANNOTATIONS_WITH_PARAMETERS_NOT_TO_BE_WRAPPED_PROPERTY_TYPE: PropertyType<Vec<String>> = PropertyType {
    name: "ktlint_annotation_handle_annotations_with_parameters_same_as_annotations_without_parameters",
    description: "Handle listed annotations identical to annotations without parameters. Value is a comma separated list of names \
                  without the '@' prefix. Use '*' for all annotations with parameters.",
    parser: comma_separated_list_value_parser,
    possible_values: &[],
    lower_casing: true,
};

pub static ANNOTATIONS_WITH_PARAMETERS_NOT_TO_BE_WRAPPED_PROPERTY: LazyLock<EditorConfigProperty<Vec<String>>> =
    LazyLock::new(|| EditorConfigProperty::new(&ANNOTATIONS_WITH_PARAMETERS_NOT_TO_BE_WRAPPED_PROPERTY_TYPE, vec!["unset".to_owned()]));

pub struct AnnotationRule {
    code_style: CodeStyleValue,
    indent_config: IndentConfig,
    annotations_with_parameters_no_to_be_wrapped: Vec<String>,
    ktlint_version: KtlintVersion,
}

impl AnnotationRule {
    pub fn new() -> AnnotationRule {
        AnnotationRule {
            code_style: CODE_STYLE_PROPERTY.default_value,
            indent_config: IndentConfig::default_indent_config(),
            annotations_with_parameters_no_to_be_wrapped: ANNOTATIONS_WITH_PARAMETERS_NOT_TO_BE_WRAPPED_PROPERTY.default_value.clone(),
            ktlint_version: KtlintVersion::default(),
        }
    }

    /// Upstream's `lazy`, first read after `beforeFirstNode`.
    fn handle_all_annotations_with_parameters_same_as_annotations_without_parameters(&self) -> bool {
        self.annotations_with_parameters_no_to_be_wrapped.iter().any(|it| it == "*")
    }
}

impl Default for AnnotationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for AnnotationRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:annotation")
    }

    fn visitor_modifiers(&self) -> &'static [VisitorModifier] {
        const MODIFIERS: &[VisitorModifier] = &[VisitorModifier::run_after("standard:enum-wrapping")];
        MODIFIERS
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
            PropertyRef::from(&*ANNOTATIONS_WITH_PARAMETERS_NOT_TO_BE_WRAPPED_PROPERTY),
        ]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.code_style = editor_config.get(&CODE_STYLE_PROPERTY);
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        self.annotations_with_parameters_no_to_be_wrapped = editor_config.get(&ANNOTATIONS_WITH_PARAMETERS_NOT_TO_BE_WRAPPED_PROPERTY);
        self.ktlint_version = KtlintVersion::of(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        match ast.element_type(node) {
            FILE_ANNOTATION_LIST => {
                self.visit_annotation_list(ast, node, emit);
                entries::visit_file_annotation_list(ast, node, emit);
            }
            ANNOTATED_EXPRESSION | MODIFIER_LIST => self.visit_annotation_list(ast, node, emit),
            // Annotation array
            //     @[...]
            ANNOTATION => entries::visit_annotation(ast, node, emit),
            ANNOTATION_ENTRY => entries::visit_annotation_entry(ast, node, emit),
            TYPE_ARGUMENT_LIST => self.visit_type_argument_list(ast, node, emit),
            _ => {}
        }
    }
}

impl AnnotationRule {
    fn visit_annotation_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        assert!(ANNOTATION_CONTAINER.contains(&ast.element_type(node)), "{FAILED_REQUIREMENT}");

        if !self.should_wrap_annotations(ast, node) {
            return;
        }
        let expected_indent = if ast.element_type(node) == ANNOTATED_EXPRESSION {
            self.indent_config.sibling_indent_of(ast, node)
        } else if self.has_annotation_before_constructor(ast, node) {
            self.indent_config.sibling_indent_of(ast, ast.parent(node).expect("NullPointerException: parent!!"))
        } else {
            self.indent_config.parent_indent_of(ast, node)
        };

        // A lazy walk of the live children, as upstream's `children.filter { }.forEachIndexed { }`.
        let mut index = 0;
        let mut child = ast.first_child_node(node);
        while let Some(annotation_entry) = child {
            if ast.element_type(annotation_entry) == ANNOTATION_ENTRY
                && (self.is_annotation_entry_with_value_argument_list_that_should_be_wrapped(ast, annotation_entry)
                    || !self.is_preceded_by_other_annotation_entry_without_parameters_on_the_same_line(ast, annotation_entry))
            {
                let prev_leaf = ast.prev_leaf(annotation_entry);
                // Allow in ktlint_official code style:
                //     class Foo(
                //         bar: Bar,
                //     ) : @Suppress("DEPRECATION")
                //         FooBar()
                let allowed = index == 0
                    && self.code_style == CodeStyleValue::KtlintOfficial
                    && entries::annotation_on_same_line_as_closing_parenthesis_of_class_parameter_list(ast, prev_leaf);
                if let Some(prev_leaf) = prev_leaf.filter(|&it| !allowed && !ast.is_white_space_with_newline(it)) {
                    emit(ast, ast.start_offset(prev_leaf), "Expected newline before annotation", true).if_autocorrect_allowed(|| {
                        // Let the indentation rule determine the exact indentation and only report and fix when the line needs
                        // to be wrapped
                        let text = ast.text(prev_leaf);
                        let before_last_newline = text.rfind('\n').map_or("", |i| &text[..i]);
                        ast.upsert_whitespace_before_me(prev_leaf, &format!("{before_last_newline}{expected_indent}"));
                    });
                }
                index += 1;
            }
            child = ast.next_sibling(annotation_entry);
        }

        if let Some(prev_leaf) = ast
            .children(node)
            .filter(|&it| ast.element_type(it) == ANNOTATION_ENTRY)
            .last()
            .map(|it| ast.last_child_leaf_or_self(it))
            .and_then(|it| ast.next_code_leaf(it))
            .and_then(|it| ast.prev_leaf(it))
            .filter(|&it| !ast.is_white_space_with_newline(it))
        {
            // Let the indentation rule determine the exact indentation and only report and fix when the line needs to be wrapped
            emit(ast, ast.start_offset(prev_leaf), "Expected newline after last annotation", true)
                .if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(prev_leaf, &expected_indent));
        }

        if let Some(leaf) = Some(node)
            .filter(|&it| ast.element_type(it) == ANNOTATED_EXPRESSION)
            .filter(|&it| ast.next_code_sibling(it).map(|s| ast.element_type(s)) != Some(OPERATION_REFERENCE))
            .map(|it| ast.last_child_leaf_or_self(it))
            .and_then(|it| ast.next_code_leaf(it))
            .and_then(|it| ast.prev_leaf(it))
            .filter(|&it| !ast.is_white_space_with_newline(it))
        {
            // Let the indentation rule determine the exact indentation and only report and fix when the line needs to be wrapped
            emit(ast, ast.start_offset(leaf), "Expected newline", true).if_autocorrect_allowed(|| {
                let indent = ast.indent(node);
                ast.upsert_whitespace_before_me(leaf, &indent);
            });
        }
    }

    /// 1.8 also wraps an annotated expression before a lambda (#3268).
    fn should_wrap_annotations(&self, ast: &Ast, n: NodeId) -> bool {
        if !self.ktlint_version.is_1_8() && is_annotated_expression_before_lambda_expression(ast, n) {
            false
        } else {
            self.has_annotation_with_parameter(ast, n)
                || has_multiple_annotations_on_same_line(ast, n)
                || self.has_annotation_before_constructor(ast, n)
        }
    }

    fn has_annotation_with_parameter(&self, ast: &Ast, n: NodeId) -> bool {
        assert!(ANNOTATION_CONTAINER.contains(&ast.element_type(n)), "{FAILED_REQUIREMENT}");
        ast.children(n).any(|it| {
            let grand_parent_type = ast.parent(it).and_then(|p| ast.parent(p)).map(|p| ast.element_type(p));
            self.is_annotation_entry_with_value_argument_list_that_should_be_wrapped(ast, it)
                && grand_parent_type != Some(VALUE_PARAMETER)
                && grand_parent_type != Some(VALUE_ARGUMENT)
                && entries::is_not_receiver_target_annotation(ast, it)
        })
    }

    fn has_annotation_before_constructor(&self, ast: &Ast, n: NodeId) -> bool {
        self.code_style == CodeStyleValue::KtlintOfficial
            && has_annotation_entry(ast, n)
            && ast.next_code_sibling(n).map(|it| ast.element_type(it)) == Some(CONSTRUCTOR_KEYWORD)
    }

    fn visit_type_argument_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let should_wrap = ast
            .children(node)
            .filter(|&it| ast.element_type(it) == TYPE_PROJECTION)
            .filter_map(|it| ast.find_child_by_type(it, TYPE_REFERENCE))
            .filter_map(|it| ast.find_child_by_type(it, MODIFIER_LIST))
            .any(|it| self.should_wrap_annotations(ast, it));
        if should_wrap {
            self.wrap_type_argument_list(ast, node, emit);
        }
    }

    fn wrap_type_argument_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let type_projections: Vec<NodeId> = ast.children(node).filter(|&it| ast.element_type(it) == TYPE_PROJECTION).collect();
        for type_projection in type_projections {
            let prev_leaf = ast.prev_leaf(type_projection).filter(|&it| ast.is_white_space(it));
            if prev_leaf.is_none_or(|it| ast.is_white_space_without_newline(it)) {
                emit(ast, ast.start_offset(type_projection) - 1, "Expected newline", true).if_autocorrect_allowed(|| {
                    let indent = self.indent_config.child_indent_of(ast, node);
                    ast.upsert_whitespace_before_me(type_projection, &indent);
                });
            }
        }

        if let Some(gt) = ast.find_child_by_type(node, GT) {
            let prev_leaf = ast.prev_leaf(gt).filter(|&it| ast.is_white_space(it));
            if prev_leaf.is_none_or(|it| ast.is_white_space_without_newline(it)) {
                emit(ast, ast.start_offset(gt), "Expected newline", true).if_autocorrect_allowed(|| {
                    let indent = self.indent_config.child_indent_of(ast, node);
                    ast.upsert_whitespace_before_me(gt, &indent);
                });
            }
        }
    }
}

fn is_annotated_expression_before_lambda_expression(ast: &Ast, n: NodeId) -> bool {
    Some(n)
        .filter(|&it| ast.element_type(it) == ANNOTATED_EXPRESSION)
        .and_then(|it| ast.find_child_by_type(it, ANNOTATION_ENTRY))
        .is_some_and(|it| ast.next_code_sibling(it).map(|s| ast.element_type(s)) == Some(LAMBDA_EXPRESSION))
}

fn has_multiple_annotations_on_same_line(ast: &Ast, n: NodeId) -> bool {
    assert!(ANNOTATION_CONTAINER.contains(&ast.element_type(n)), "{FAILED_REQUIREMENT}");
    ast.children(n).any(|it| {
        let parent_type = ast.parent(it).map(|p| ast.element_type(p));
        let grand_parent_type = ast.parent(it).and_then(|p| ast.parent(p)).map(|p| ast.element_type(p));
        parent_type != Some(ANNOTATION)
            && grand_parent_type != Some(VALUE_PARAMETER)
            && grand_parent_type != Some(VALUE_ARGUMENT)
            && entries::is_preceded_by_other_annotation_entry_on_the_same_line(ast, it)
            && entries::is_last_annotation_entry(ast, it)
        // Code below is disallowed
        //   @Foo1 @Foo2 fun foo() {}
        // But following is allowed:
        //   @[Foo1 Foo2] fun foo() {}
        //   fun foo(@Bar1 @Bar2 bar) {}
    })
}

fn has_annotation_entry(ast: &Ast, n: NodeId) -> bool {
    ast.children(n).any(|it| ast.element_type(it) == ANNOTATION_ENTRY)
}
