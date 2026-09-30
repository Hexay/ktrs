//! Port of ktlint-rule-engine-core `IndentConfig.kt`, the members the ported rules use.

use ktrs_ast::{Ast, NodeId};

use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{EditorConfig, IndentStyle};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndentConfig {
    pub indent_style: IndentStyle,
    pub tab_width: i32,
    /// One indent level.
    pub indent: String,
}

impl IndentConfig {
    pub fn new(indent_style: IndentStyle, tab_width: i32) -> IndentConfig {
        let indent = if tab_width <= 0 {
            String::new()
        } else {
            match indent_style {
                IndentStyle::Tab => "\t".to_owned(),
                IndentStyle::Space => " ".repeat(tab_width as usize),
            }
        };
        IndentConfig { indent_style, tab_width, indent }
    }

    pub fn default_indent_config() -> IndentConfig {
        let defaults = EditorConfig::default();
        IndentConfig::new(defaults.indent_style, defaults.indent_size)
    }

    pub fn disabled(&self) -> bool {
        self.tab_width <= 0
    }

    pub fn child_indent_of(&self, ast: &Ast, node: NodeId) -> String {
        ast.indent(node) + &self.indent
    }

    pub fn sibling_indent_of(&self, ast: &Ast, node: NodeId) -> String {
        self.parent_indent_of(ast, node) + &self.indent
    }

    pub fn parent_indent_of(&self, ast: &Ast, node: NodeId) -> String {
        ast.indent(ast.parent(node).expect("NullPointerException: parentIndentOf(root)"))
    }
}
