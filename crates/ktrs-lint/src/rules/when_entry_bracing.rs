//! Port of ktlint-ruleset-standard `WhenEntryBracing.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{ARROW, BLOCK, WHEN, WHEN_ENTRY, WHITE_SPACE};

use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, KtlintVersion, PropertyRef};
use crate::engine::kotlin_text::trim_margin;
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[WHEN]);

/// If any when condition is using curly braces, then all other when conditions should use braces as well.
pub struct WhenEntryBracing {
    indent_config: IndentConfig,
    ktlint_version: KtlintVersion,
}

impl WhenEntryBracing {
    pub fn new() -> WhenEntryBracing {
        WhenEntryBracing { indent_config: IndentConfig::default_indent_config(), ktlint_version: KtlintVersion::default() }
    }
}

impl Default for WhenEntryBracing {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for WhenEntryBracing {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:when-entry-bracing")
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
        self.ktlint_version = KtlintVersion::of(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == WHEN {
            self.visit_when_statement(ast, node, emit);
        }
    }
}

impl WhenEntryBracing {
    fn visit_when_statement(&self, ast: &mut Ast, node: NodeId, emit_and_approve: &mut Emit<'_>) {
        if has_any_when_entry_with_block_after_arrow(ast, node) || has_any_when_entry_with_multiline_body(ast, node) {
            self.add_braces_to_when_entry(ast, node, emit_and_approve);
        }
    }

    fn add_braces_to_when_entry(&self, ast: &mut Ast, node: NodeId, emit_and_approve: &mut Emit<'_>) {
        // A lazy walk of the live children: after a rewrite it continues inside the dummy holder of the removed entry.
        let mut child = ast.first_child_node(node);
        while let Some(when_entry) = child {
            if ast.element_type(when_entry) == WHEN_ENTRY
                && !has_block_after_arrow(ast, when_entry)
                && let Some(arrow) = ast.find_child_by_type(when_entry, ARROW)
            {
                let non_white_space_sibling = ast.next_sibling_matching(arrow, |it| !ast.is_white_space(it)).unwrap_or(arrow);
                emit_and_approve(
                    ast,
                    ast.start_offset(non_white_space_sibling),
                    "Body of when entry should be surrounded by braces if any when entry body is surrounded by braces \
                     or has a multiline body",
                    true,
                )
                .if_autocorrect_allowed(|| {
                    if self.ktlint_version.is_1_8() {
                        self.surround_with_braces_1_8(ast, arrow);
                    } else {
                        self.surround_with_braces(ast, arrow);
                    }
                });
            }
            child = ast.next_sibling(when_entry);
        }
    }

    fn surround_with_braces(&self, ast: &mut Ast, arrow: NodeId) {
        assert!(ast.element_type(arrow) == ARROW, "IllegalArgumentException: Failed requirement.");
        let parent_indent = self.indent_config.parent_indent_of(ast, arrow);
        let when_entry_indent = parent_indent.strip_prefix('\n').unwrap_or(&parent_indent).to_owned();
        let when_entry = ast.parent(arrow).expect("NullPointerException: parent!!");
        // Find the anchor node, e.g. the last node before the current when-entry which is not altered
        let sibling_before_when_entry = ast.prev_sibling(when_entry).expect("NullPointerException: prevSibling!!");
        // Find the first leaf after the when-entry (including the EOL comment after it) that not has to be enclosed within the braces
        let last = ast
            .siblings(sibling_before_when_entry, true)
            .take_while(|&it| !ast.is_white_space_with_newline(it))
            .last()
            .expect("NoSuchElementException: Sequence is empty.");
        let stop_leaf = ast.first_child_leaf_or_self(ast.next_sibling(last).expect("NullPointerException: nextSibling!!"));
        let prev_code_sibling = ast.prev_code_sibling(arrow).expect("NullPointerException: prevCodeSibling!!");
        let body: String = ast
            .leaves(arrow, true)
            .skip_while(|&it| ast.is_white_space(it))
            .take_while(|&it| it != stop_leaf)
            .map(|it| ast.leaf_text(it).to_owned())
            .collect();
        // Replace the whitespaces (possibly this could be a proper indent) at the beginning of the body with an indent. In case
        // the body was already a multiline statement, then the second and following lines should already be properly indented.
        let text = format!(
            "{when_entry_indent}{} -> {{{}{body}\n{when_entry_indent}}}",
            ast.text(prev_code_sibling),
            self.indent_config.child_indent_of(ast, arrow),
        );
        let when_entry_node = create_when_entry_node(ast, &text);
        // Remove the old when entry and if applicable the EOL-comment after it
        ast.remove_range(when_entry, when_entry, Some(stop_leaf));
        if let Some(parent) = ast.parent(sibling_before_when_entry) {
            ast.add_child(parent, when_entry_node.expect("NullPointerException: whenEntryNode!!"), Some(stop_leaf));
        }
    }

    /// 1.8's `surroundWithBraces`, before #3261: the body after the arrow becomes a block in place, so every
    /// condition stays (2.0 rebuilds the entry from the last condition only).
    fn surround_with_braces_1_8(&self, ast: &mut Ast, arrow: NodeId) {
        assert!(ast.element_type(arrow) == ARROW, "IllegalArgumentException: Failed requirement.");
        let parent_indent = self.indent_config.parent_indent_of(ast, arrow);
        let when_entry_indent = parent_indent.strip_prefix('\n').unwrap_or(&parent_indent).to_owned();
        let body: String =
            ast.siblings(arrow, true).skip_while(|&it| ast.is_white_space(it)).map(|it| ast.text(it)).collect();
        let text = format!(
            "{when_entry_indent}true -> {{{}{body}\n{when_entry_indent}}}",
            self.indent_config.child_indent_of(ast, arrow),
        );
        let block_expression =
            create_when_entry_node(ast, &text).and_then(|when_entry| ast.find_child_by_type(when_entry, BLOCK));
        if let Some(when_entry) = ast.parent(arrow) {
            let next_sibling = ast.next_sibling(arrow).expect("NullPointerException: nextSibling!!");
            ast.remove_range(when_entry, next_sibling, None);
        }
        let prev_sibling = ast.prev_sibling(arrow).expect("NullPointerException: prevSibling!!");
        if let Some(when_entry) = ast.parent(prev_sibling) {
            let white_space = ast.new_leaf(WHITE_SPACE, " ");
            ast.add_child(when_entry, white_space, None);
            ast.add_child(when_entry, block_expression.expect("NullPointerException: blockExpression!!"), None);
        }
    }
}

fn has_any_when_entry_with_block_after_arrow(ast: &Ast, n: NodeId) -> bool {
    ast.children(n).any(|it| ast.element_type(it) == WHEN_ENTRY && has_block_after_arrow(ast, it))
}

fn has_block_after_arrow(ast: &Ast, n: NodeId) -> bool {
    assert!(ast.element_type(n) == WHEN_ENTRY, "IllegalArgumentException: Failed requirement.");
    ast.find_child_by_type(n, ARROW).is_some_and(|arrow| ast.siblings(arrow, true).any(|it| ast.element_type(it) == BLOCK))
}

fn has_any_when_entry_with_multiline_body(ast: &Ast, n: NodeId) -> bool {
    ast.children(n).any(|it| ast.element_type(it) == WHEN_ENTRY && has_multiline_body(ast, it))
}

fn has_multiline_body(ast: &Ast, n: NodeId) -> bool {
    assert!(ast.element_type(n) == WHEN_ENTRY, "IllegalArgumentException: Failed requirement.");
    ast.find_child_by_type(n, ARROW).is_some_and(|arrow| ast.siblings(arrow, true).any(|it| ast.is_white_space_with_newline(it)))
}

fn create_when_entry_node(ast: &mut Ast, when_entry: &str) -> Option<NodeId> {
    // Upstream trims the margin after interpolating, so `|` margins inside the entry's own raw strings go too.
    let text = trim_margin(&format!("\n|when {{\n|{when_entry}\n|}}\n"));
    ast.create_ast_node_from_text(&text)
        .and_then(|it| ast.find_child_by_type(it, WHEN))
        .and_then(|it| ast.find_child_by_type(it, WHEN_ENTRY))
}
