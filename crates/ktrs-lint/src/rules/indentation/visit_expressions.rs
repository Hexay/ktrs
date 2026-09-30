//! `IndentationRule.kt` from `visitBinaryExpression` to `visitTryCatchFinally`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ANNOTATION, ARROW, BINARY_EXPRESSION, BLOCK, BODY, CATCH, CONDITION, ELVIS, EQ, FINALLY, KDOC_START, LBRACE, LPAR,
    OPERATION_REFERENCE, RBRACKET, RPAR,
};

use super::{Args, IndentationRule, KDOC_CONTINUATION_INDENT, TYPE_CONSTRAINT_CONTINUATION_INDENT};
use crate::ast_node_extension::AstNodeExtension;

impl IndentationRule {
    pub(super) fn visit_binary_expression(&mut self, ast: &Ast, node: NodeId) {
        if is_part_of_binary_expression_wrapped_in_condition(ast, node) {
            // Complex binary expression are nested in such a way that the indent context of the condition
            // wrapper is not the last node on the stack
            let condition_indent_context = self
                .indent_context_stack
                .iter()
                .rev()
                .find(|it| ast.element_type(it.from_ast_node) != BINARY_EXPRESSION)
                .unwrap_or_else(|| panic!("NoSuchElementException: List contains no element matching the predicate."));
            let (node_indent, child_indent) = (condition_indent_context.node_indent.clone(), condition_indent_context.child_indent.clone());
            // Create new indent context for the remainder (operator and right-hand side) of the binary expression
            let operation_reference = ast.find_child_by_type(node, OPERATION_REFERENCE).expect("NullPointerException: findChildByType!!");
            self.start_indent_context(
                ast,
                operation_reference,
                Args::default().to(ast.last_child_leaf_or_self(node)).node_indent(node_indent).child_indent(child_indent),
            );
            let first_child_node = ast.first_child_node(node).expect("NullPointerException: firstChildNode");
            self.start_indent_context(ast, first_child_node, Args::default());
        } else if ast.parent(node).map(|it| ast.element_type(it)) != Some(BINARY_EXPRESSION)
            || ast
                .find_child_by_type(node, OPERATION_REFERENCE)
                .and_then(|it| ast.first_child_node(it))
                .map(|it| ast.element_type(it))
                == Some(ELVIS)
        {
            self.start_indent_context(ast, node, Args::default());
        }
    }

    pub(super) fn visit_identifier_in_property(&mut self, ast: &Ast, node: NodeId) {
        let parent = ast.parent(node).expect("NullPointerException: parent!!");
        self.start_indent_context(ast, node, Args::default().to(ast.last_child_leaf_or_self(parent)));
    }

    pub(super) fn visit_when(&mut self, ast: &Ast, node: NodeId) {
        let mut next_to_ast_node = ast.last_child_leaf_or_self(node);
        if let Some(lpar) = ast.find_child_by_type(node, LPAR) {
            let rpar = ast.find_child_by_type(node, RPAR).expect("NullPointerException: findChildByType(RPAR)!!");
            next_to_ast_node = self.start_indent_context(ast, lpar, Args::default().to(rpar).last_child_indent("")).prev_code_leaf(ast);
        }
        self.start_indent_context(ast, node, Args::default().to(next_to_ast_node));
    }

    pub(super) fn visit_when_entry(&mut self, ast: &Ast, node: NodeId) {
        let Some(arrow) = ast.find_child_by_type(node, ARROW) else { return };
        let prev_sibling = ast.prev_sibling_matching(arrow, |it| !ast.is_part_of_comment(it));
        let last_leaf = ast.last_child_leaf_or_self(node);
        match prev_sibling {
            Some(prev_sibling) if self.indent_when_arrow_on_new_line && ast.is_white_space_with_newline(prev_sibling) => {
                if ast.next_code_sibling(arrow).map(|it| ast.element_type(it)) == Some(BLOCK) && !self.is_official() {
                    // Uglify the indentation to below to keep compatible with default formatting Intellij IDEA
                    //     val foo =
                    //        when (bar()) {
                    //            is Bar1
                    //                -> {
                    //                "bar1"
                    //            }
                    //        }
                    let indent = self.indent_config.indent.clone();
                    self.start_indent_context(ast, prev_sibling, Args::default().to(last_leaf).first_child_indent(indent).child_indent(""));
                } else {
                    // Reformat to below. This is not compatible with default IDEA formatting. But the closing brace is now at
                    // least aligned consistently.
                    //     val foo =
                    //        when (bar()) {
                    //            is Bar1
                    //                -> {
                    //                    "bar1"
                    //                }
                    //        }
                    let after_arrow = ast.next_leaf(arrow).expect("NullPointerException: arrow.nextLeaf!!");
                    self.start_indent_context(ast, after_arrow, Args::default().to(last_leaf));
                    let to = ast.prev_leaf(prev_sibling).expect("NullPointerException: prevSibling.prevLeaf!!");
                    self.start_indent_context(ast, node, Args::default().to(to).child_indent(""));
                }
            }
            _ => {
                let after_arrow = ast.next_leaf(arrow).expect("NullPointerException: arrow.nextLeaf!!");
                self.start_indent_context(ast, after_arrow, Args::default().to(last_leaf));
                self.start_indent_context(ast, node, Args::default().to(arrow).child_indent(""));
            }
        }
    }

    pub(super) fn visit_where_keyword_before_type_constraint_list(&mut self, ast: &Ast, node: NodeId) {
        if ast.prev_leaf(node).is_some_and(|it| !ast.is_white_space_with_newline(it))
            && !self.indent_context_stack.last().expect("NullPointerException: peekLast().activated").activated
        {
            self.indent_context_stack.last_mut().unwrap().activated = true;
        }

        let to = ast.last_child_leaf_or_self(ast.next_code_sibling(node).expect("NullPointerException: nextCodeSibling!!"));
        self.start_indent_context(ast, node, Args::default().to(to).child_indent(TYPE_CONSTRAINT_CONTINUATION_INDENT));
    }

    pub(super) fn visit_kdoc(&mut self, ast: &Ast, node: NodeId) {
        if let Some(from_ast_node) = ast.find_child_by_type(node, KDOC_START).and_then(|it| ast.next_leaf(it)) {
            let to = ast.last_child_leaf_or_self(node);
            self.start_indent_context(ast, from_ast_node, Args::default().to(to).child_indent(KDOC_CONTINUATION_INDENT));
        }
    }

    pub(super) fn visit_property_accessor(&mut self, ast: &Ast, node: NodeId) {
        let mut next_to_ast_node = ast.last_child_leaf_or_self(node);
        if let Some(from_ast_node) = ast.find_child_by_type(node, EQ) {
            let to = ast.last_child_leaf_or_self(node);
            next_to_ast_node = self.start_indent_context(ast, from_ast_node, Args::default().to(to)).prev_code_leaf(ast);
        }
        // No indent on preceding annotations and comments
        self.start_indent_context(ast, node, Args::default().to(next_to_ast_node).child_indent(""));
    }

    pub(super) fn visit_conditional_loop(&mut self, ast: &Ast, node: NodeId) {
        // Inner indent contexts in reversed order
        if let Some(rpar) = ast
            .find_child_by_type(node, BODY)
            .filter(|&it| ast.next_code_leaf(it).map(|l| ast.element_type(l)) != Some(LBRACE))
        {
            self.start_indent_context(ast, rpar, Args::default().to(ast.last_child_leaf_or_self(node)));
        }
        if let Some(from_ast_node) = ast.find_child_by_type(node, CONDITION) {
            self.start_indent_context(ast, from_ast_node, Args::default());
        }
    }

    pub(super) fn visit_l_bracket(&mut self, ast: &Ast, node: NodeId) {
        // Should be resolved in IntelliJ IDEA default formatter:
        // https://youtrack.jetbrains.com/issue/KTIJ-14859/Too-little-indentation-inside-the-brackets-in-multiple-annotations-with-the-same-target
        if let Some(rbracket) = ast
            .parent(node)
            .filter(|&it| ast.element_type(it) != ANNOTATION)
            .and_then(|it| ast.find_child_by_type(it, RBRACKET))
        {
            self.start_indent_context(ast, node, Args::default().to(rbracket).first_child_indent("").last_child_indent(""));
        }
    }

    pub(super) fn visit_nullable_type(&mut self, ast: &Ast, node: NodeId) {
        // Inner indent contexts in reversed order
        let mut next_to_ast_node = ast.last_child_leaf_or_self(node);
        if let Some(from_ast_node) = ast.find_child_by_type(node, RPAR) {
            next_to_ast_node =
                self.start_indent_context(ast, from_ast_node, Args::default().to(next_to_ast_node).child_indent("")).prev_code_leaf(ast);
        }
        self.start_indent_context(ast, node, Args::default().to(next_to_ast_node));
    }

    pub(super) fn visit_destructuring_declaration(&mut self, ast: &Ast, node: NodeId) {
        // Inner indent contexts in reversed order
        let mut next_to_ast_node = ast.last_child_leaf_or_self(node);
        if let Some(eq) = ast.find_child_by_type(node, EQ) {
            next_to_ast_node = self.start_indent_context(ast, eq, Args::default().to(next_to_ast_node)).prev_code_leaf(ast);
        }

        if ast.find_child_by_type(node, RPAR).is_some() {
            self.start_indent_context(ast, node, Args::default().to(next_to_ast_node).last_child_indent(""));
        }
    }

    pub(super) fn visit_try_catch_finally(&mut self, ast: &Ast, node: NodeId) {
        // Inner indent contexts in reversed order
        let mut next_to_ast_node = ast.last_child_leaf_or_self(node);
        if let Some(finally) = ast.find_child_by_type(node, FINALLY) {
            next_to_ast_node = self.start_indent_context(ast, finally, Args::default().to(next_to_ast_node).child_indent("")).prev_code_leaf(ast);
        }

        if let Some(catch) = ast.find_child_by_type(node, CATCH) {
            next_to_ast_node = self.start_indent_context(ast, catch, Args::default().to(next_to_ast_node).child_indent("")).prev_code_leaf(ast);
        }

        self.start_indent_context(ast, node, Args::default().to(next_to_ast_node).last_child_indent(""));
    }
}

use super::context::is_part_of_binary_expression_wrapped_in_condition;
