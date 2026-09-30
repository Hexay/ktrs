//! `IndentationRule.kt` from `visitValueArgument` to `visitObjectDeclaration`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ARROW, BLOCK, CALL_EXPRESSION, COLON, CONSTRUCTOR_DELEGATION_CALL, CONSTRUCTOR_KEYWORD, ELSE, EQ, FUNCTION_LITERAL,
    PRIMARY_CONSTRUCTOR, RBRACE, RPAR, SUPER_TYPE_LIST, THEN, TYPE_CONSTRAINT_LIST, TYPE_REFERENCE, VALUE_PARAMETER,
    VALUE_PARAMETER_LIST, WHERE_KEYWORD,
};

use super::{Args, IndentationRule};
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines};
use crate::editorconfig::CodeStyleValue;

impl IndentationRule {
    pub(super) fn visit_value_argument(&mut self, ast: &Ast, node: NodeId) {
        if self.is_official() || self.code_style == CodeStyleValue::AndroidStudio {
            // Deviate from standard IntelliJ IDEA formatting to allow formatting below:
            //     val foo = foo(
            //         parameterName =
            //             "The quick brown fox "
            //                .plus("jumps ")
            //                .plus("over the lazy dog"),
            //     )
            self.start_indent_context(ast, node, Args::default().last_child_indent(""));
        }
    }

    pub(super) fn visit_secondary_constructor(&mut self, ast: &Ast, node: NodeId) {
        if let Some(constructor_delegation_call) = ast.find_child_by_type(node, CONSTRUCTOR_DELEGATION_CALL) {
            let from_ast_node = skip_leading_whitespace_comments_and_annotations(ast, node);
            let next_to_ast_node = self.start_indent_context(ast, constructor_delegation_call, Args::default()).prev_code_leaf(ast);

            // Leading annotations and comments should be indented at same level as constructor itself
            if Some(from_ast_node) != ast.next_leaf(node) {
                self.start_indent_context(ast, node, Args::default().to(next_to_ast_node).child_indent(""));
            }
        }
    }

    pub(super) fn visit_if(&mut self, ast: &Ast, node: NodeId) {
        let mut next_to_ast_node = ast.last_child_leaf_or_self(node);
        if let Some(from_ast_node) = ast.find_child_by_type(node, ELSE) {
            next_to_ast_node = self.start_indent_context(ast, from_ast_node, Args::default().to(next_to_ast_node)).prev_code_leaf(ast);
        }

        if let Some(node_after_then_block) = ast.find_child_by_type(node, THEN).map(|it| ast.last_child_leaf_or_self(it)).and_then(|it| ast.next_leaf(it)) {
            next_to_ast_node = self
                .start_indent_context(ast, node_after_then_block, Args::default().to(next_to_ast_node).child_indent(""))
                .prev_code_leaf(ast);
        }
        if let Some(node_after_condition_block) = ast.find_child_by_type(node, RPAR).and_then(|it| ast.next_code_leaf(it)) {
            next_to_ast_node =
                self.start_indent_context(ast, node_after_condition_block, Args::default().to(next_to_ast_node)).prev_code_leaf(ast);
        }
        // No indent for the RPAR
        self.start_indent_context(ast, node, Args::default().to(next_to_ast_node).last_child_indent(""));
    }

    pub(super) fn visit_lbrace(&mut self, ast: &Ast, node: NodeId) {
        // Outer indent context
        let rbrace = ast
            .next_sibling_matching(node, |it| ast.element_type(it) == RBRACE)
            .unwrap_or_else(|| panic!("IllegalArgumentException: Can not find matching rbrace"));
        self.start_indent_context(ast, node, Args::default().to(rbrace).first_child_indent("").last_child_indent(""));

        // Inner indent context in reversed order
        if let Some(arrow) =
            ast.parent(node).filter(|&it| ast.element_type(it) == FUNCTION_LITERAL).and_then(|it| ast.find_child_by_type(it, ARROW))
        {
            self.start_indent_context(ast, arrow, Args::default().to(rbrace).last_child_indent(""));
            let to = ast.prev_code_leaf(arrow).expect("NullPointerException: arrow.prevCodeLeaf!!");
            let child_indent = self.calculate_indent_of_function_literal_parameters(ast, arrow);
            self.start_indent_context(ast, node, Args::default().to(to).child_indent(child_indent));
        }
    }

    fn calculate_indent_of_function_literal_parameters(&self, ast: &Ast, this: NodeId) -> String {
        if is_first_parameter_of_function_literal_preceded_by_new_line(ast, this) {
            if self.is_official() {
                // Indent with single indent as defined in Kotlin Coding conventions
                self.indent_config.indent.clone()
            } else {
                // Comply with default IDEA formatting although it is not compliant with Kotlin Coding conventions
                // val fieldExample =
                //      LongNameClass {
                //              paramA,
                //              paramB,
                //              paramC ->
                //          ClassB(paramA, paramB, paramC)
                //      }
                self.indent_config.indent.repeat(2)
            }
        } else {
            // Allow default IntelliJ IDEA formatting:
            // val fieldExample =
            //     LongNameClass { paramA,
            //                     paramB,
            //                     paramC ->
            //         ClassB(paramA, paramB, paramC)
            //     }
            // val fieldExample =
            //     someFunction(
            //         1,
            //         2,
            //     ) { paramA,
            //         paramB,
            //         paramC ->
            //         ClassB(paramA, paramB, paramC)
            //     }
            Some(this)
                .filter(|&it| ast.is_part_of(it, CALL_EXPRESSION))
                .and_then(|it| ast.parent(it))
                .map(|parent| {
                    let length: usize = ast
                        .leaves(parent, false)
                        .take_while(|&it| !ast.is_white_space_with_newline(it))
                        .map(|it| ast.text_length_utf16(it))
                        .sum();
                    // need to add spaces to compensate for "{ "
                    " ".repeat(length + 2)
                })
                .unwrap_or_else(|| self.indent_config.indent.repeat(2))
        }
    }

    pub(super) fn visit_lpar_before_condition(&mut self, ast: &Ast, node: NodeId) {
        // Allow to pickup whitespace before condition
        let from_ast_node = ast.next_leaf(node).unwrap_or_else(|| panic!("IllegalArgumentException: Required value was null."));
        // Ignore whitespace after condition but before rpar
        let to_ast_node = ast.last_child_leaf_or_self(
            ast.next_code_sibling(node).unwrap_or_else(|| panic!("IllegalArgumentException: Required value was null.")),
        );
        let node_indent = self.current_indent() + &self.indent_config.indent;
        self.start_indent_context(ast, from_ast_node, Args::default().to(to_ast_node).node_indent(node_indent).child_indent(""));
    }

    pub(super) fn visit_value_parameter(&mut self, ast: &Ast, node: NodeId) {
        // Inner indent contexts in reversed order
        let mut next_to_ast_node = ast.last_child_leaf_or_self(node);
        if let Some(from_ast_node) = ast.find_child_by_type(node, EQ) {
            next_to_ast_node = self.start_indent_context(ast, from_ast_node, Args::default().to(next_to_ast_node)).prev_code_leaf(ast);
        }

        if self.is_official() {
            // Deviate from standard IntelliJ IDEA formatting to allow formatting below:
            //     fun process(
            //         aVariableWithAVeryLongName:
            //             TypeWithAVeryLongNameThatDoesNotFitOnSameLineAsTheVariableName
            //     ): List<Output>
            if let Some(from_ast_node) = ast.find_child_by_type(node, COLON) {
                next_to_ast_node = self.start_indent_context(ast, from_ast_node, Args::default().to(next_to_ast_node)).prev_code_leaf(ast);
            }
        }

        // Leading annotations and comments should be indented at same level as constructor itself
        let from_ast_node = skip_leading_whitespace_comments_and_annotations(ast, node);
        if Some(from_ast_node) != ast.first_child_node(node)
            && ast.prev_sibling_matching(node, |it| ast.is_white_space_with_newline(it)).is_none()
            && Some(node) == ast.parent(node).and_then(|it| ast.find_child_by_type(it, VALUE_PARAMETER))
        {
            let started = self.start_indent_context(ast, from_ast_node, Args::default().to(next_to_ast_node));
            ast.prev_leaf_matching(started.from_ast_node, |it| !ast.is_white_space(it)).expect("NullPointerException: prevLeaf!!");
        } else {
            self.start_indent_context(ast, node, Args::default().to(next_to_ast_node).child_indent(""));
        }
    }

    pub(super) fn visit_fun(&mut self, ast: &Ast, node: NodeId) {
        // Inner indent contexts in reversed order
        let mut next_to_ast_node = ast.last_child_leaf_or_self(node);
        let eq_or_block = ast.find_child_by_type(node, EQ).or_else(|| ast.find_child_by_type(node, BLOCK));
        if let Some(eq_or_block) = eq_or_block {
            next_to_ast_node = self.start_indent_context(ast, eq_or_block, Args::default().to(next_to_ast_node)).prev_code_leaf(ast);
        }

        if let Some(where_) = ast.find_child_by_type(node, WHERE_KEYWORD) {
            let type_constraint_list = ast
                .next_code_sibling(where_)
                .unwrap_or_else(|| panic!("IllegalArgumentException: Can not find code sibling after WHERE in FUN"));
            assert!(
                ast.element_type(type_constraint_list) == TYPE_CONSTRAINT_LIST,
                "IllegalArgumentException: Code sibling after WHERE in CLASS is not a TYPE_CONSTRAINT_LIST"
            );
            let child_indent = if ast.prev_leaf(where_).is_some_and(|it| ast.is_white_space_with_newline(it)) {
                self.indent_config.indent.clone()
            } else {
                let column = ast.column(where_) as i64 - 1 - ast.indent_without_newline_prefix(node).len() as i64;
                " ".repeat(column.max(0) as usize)
            };
            let from = get_preceding_leading_comments_and_whitespaces(ast, where_);
            let to = ast.last_child_leaf_or_self(type_constraint_list);
            next_to_ast_node = self.start_indent_context(ast, from, Args::default().to(to).child_indent(child_indent)).prev_code_leaf(ast);
        }

        if let Some(type_reference) = ast.find_child_by_type(node, TYPE_REFERENCE) {
            let from = get_preceding_leading_comments_and_whitespaces(ast, type_reference);
            next_to_ast_node = self.start_indent_context(ast, from, Args::default().to(next_to_ast_node)).prev_code_leaf(ast);
        }

        // Leading annotations and comments should be indented at same level as function itself
        self.start_indent_context(ast, node, Args::default().to(next_to_ast_node).child_indent(""));
    }

    pub(super) fn visit_class(&mut self, ast: &Ast, node: NodeId) {
        // Inner indent contexts in reversed order
        let mut next_to_ast_node = ast.last_child_leaf_or_self(node);
        if let Some(where_) = ast.find_child_by_type(node, WHERE_KEYWORD) {
            let type_constraint_list = ast
                .next_code_sibling(where_)
                .unwrap_or_else(|| panic!("IllegalArgumentException: Can not find code sibling after WHERE in CLASS"));
            assert!(
                ast.element_type(type_constraint_list) == TYPE_CONSTRAINT_LIST,
                "IllegalArgumentException: Code sibling after WHERE in CLASS is not a TYPE_CONSTRAINT_LIST"
            );
            let from = get_preceding_leading_comments_and_whitespaces(ast, where_);
            let to = ast.last_child_leaf_or_self(type_constraint_list);
            next_to_ast_node = self.start_indent_context(ast, from, Args::default().to(to)).prev_code_leaf(ast);
        }

        let primary_constructor = ast.find_child_by_type(node, PRIMARY_CONSTRUCTOR);
        let contains_constructor_keyword = primary_constructor.and_then(|it| ast.find_child_by_type(it, CONSTRUCTOR_KEYWORD)).is_some();
        if let Some(primary_constructor) = primary_constructor.filter(|_| self.is_official() && contains_constructor_keyword) {
            let from = get_preceding_leading_comments_and_whitespaces(ast, primary_constructor);
            next_to_ast_node = self.start_indent_context(ast, from, Args::default().to(next_to_ast_node)).prev_code_leaf(ast);
        } else if let Some(super_type_list) = ast.find_child_by_type(node, SUPER_TYPE_LIST) {
            let from = get_preceding_leading_comments_and_whitespaces(ast, super_type_list);
            let to = ast.last_child_leaf_or_self(super_type_list);
            next_to_ast_node = self.start_indent_context(ast, from, Args::default().to(to)).prev_code_leaf(ast);
        }

        // Leading annotations and comments should be indented at same level as class itself
        self.start_indent_context(ast, node, Args::default().to(next_to_ast_node).child_indent(""));
    }

    pub(super) fn visit_object_declaration(&mut self, ast: &Ast, node: NodeId) {
        // Inner indent contexts in reversed order
        let mut next_to_ast_node = ast.last_child_leaf_or_self(node);
        if let Some(super_type_list) = ast.find_child_by_type(node, SUPER_TYPE_LIST) {
            let from = get_preceding_leading_comments_and_whitespaces(ast, super_type_list);
            let to = ast.last_child_leaf_or_self(super_type_list);
            next_to_ast_node = self.start_indent_context(ast, from, Args::default().to(to)).prev_code_leaf(ast);
        }

        // Leading annotations and comments should be indented at same level as class itself
        self.start_indent_context(ast, node, Args::default().to(next_to_ast_node).child_indent(""));
    }
}

fn is_first_parameter_of_function_literal_preceded_by_new_line(ast: &Ast, this: NodeId) -> bool {
    ast.find_parent_by_type(this, FUNCTION_LITERAL)
        .and_then(|it| ast.find_child_by_type(it, VALUE_PARAMETER_LIST))
        .and_then(|it| ast.prev_sibling_matching(it, |s| ast.text_contains(s, '\n')))
        .is_some()
}

use super::context::{get_preceding_leading_comments_and_whitespaces, skip_leading_whitespace_comments_and_annotations};
