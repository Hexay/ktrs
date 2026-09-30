//! Port of ktlint-rule-engine-core `IndentConfig.kt`. Its `IndentStyle` and the editorconfig
//! `IndentStyleValue` of the secondary constructor are the one [`IndentStyle`] here.

use ktrs_ast::{Ast, NodeId};

use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY};
use crate::rule::IndentStyle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndentConfig {
    pub indent_style: IndentStyle,
    /// The number of spaces that is equivalent to one tab.
    pub tab_width: i32,
    /// One indent level: a tab, or `tab_width` spaces; empty when disabled.
    pub indent: String,
}

impl IndentConfig {
    pub fn new(indent_style: IndentStyle, tab_width: i32) -> IndentConfig {
        let mut config = IndentConfig { indent_style, tab_width, indent: String::new() };
        if !config.disabled() {
            config.indent = match indent_style {
                IndentStyle::Tab => config.indent_char().to_string(),
                IndentStyle::Space => config.indent_char().to_string().repeat(tab_width as usize),
            };
        }
        config
    }

    /// `DEFAULT_INDENT_CONFIG`.
    pub fn default_indent_config() -> IndentConfig {
        IndentConfig::new(INDENT_STYLE_PROPERTY.default_value, INDENT_SIZE_PROPERTY.default_value)
    }

    fn indent_char(&self) -> char {
        match self.indent_style {
            IndentStyle::Tab => '\t',
            IndentStyle::Space => ' ',
        }
    }

    fn unexpected_indent_char(&self) -> char {
        match self.indent_style {
            IndentStyle::Tab => ' ',
            IndentStyle::Space => '\t',
        }
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

    /// The indent after the last newline of `text`, tabs expanded (space style) or re-counted as tabs.
    pub fn to_normalized_indent(&self, text: &str) -> String {
        let indent = get_text_after_last_new_line(text);
        require_tabs_and_spaces(indent);
        match self.indent_style {
            IndentStyle::Space => self.replace_tab_with_spaces(indent),
            IndentStyle::Tab => "\t".repeat(self.indent_level_from(indent).max(0) as usize),
        }
    }

    fn replace_tab_with_spaces(&self, text: &str) -> String {
        let count = usize::try_from(self.tab_width)
            .unwrap_or_else(|_| panic!("IllegalArgumentException: Count 'n' must be non-negative, but was {}.", self.tab_width));
        text.replace('\t', &" ".repeat(count))
    }

    /// Full indent levels in the text after the last newline; leftover spaces are dropped.
    pub fn indent_level_from(&self, text: &str) -> i32 {
        let indent = get_text_after_last_new_line(text);
        require_tabs_and_spaces(indent);
        let spaces = self.replace_tab_with_spaces(indent).len() as i32;
        assert!(self.tab_width != 0, "ArithmeticException: / by zero");
        spaces / self.tab_width
    }

    pub fn contains_unexpected_indent_char(&self, indent_text: &str) -> bool {
        indent_text.contains(self.unexpected_indent_char())
    }

    /// UTF-16 index of the first unexpected indent char, or -1.
    pub fn index_of_first_unexpected_indent_char(&self, indent_text: &str) -> i32 {
        let unexpected = self.unexpected_indent_char() as u16;
        indent_text.encode_utf16().position(|c| c == unexpected).map_or(-1, |i| i as i32)
    }

    pub fn unexpected_indent_char_description(&self) -> &'static str {
        match self.indent_style {
            IndentStyle::Space => "tab",
            IndentStyle::Tab => "space",
        }
    }
}

fn get_text_after_last_new_line(text: &str) -> &str {
    text.rfind('\n').map_or(text, |index| &text[index + 1..])
}

/// `require(indent.matches(TABS_AND_SPACES))`.
fn require_tabs_and_spaces(indent: &str) {
    assert!(indent.chars().all(|c| c == ' ' || c == '\t'), "IllegalArgumentException: Failed requirement.");
}
