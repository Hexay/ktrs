//! Port of ktlint-ruleset-standard `NoUnusedImportsRule.kt`. Opt-in (`OnlyWhenEnabledInEditorconfig`), as an
//! import is occasionally falsely marked as unused (ktlint#3038); it ignores ktlint suppressions, so imports only
//! used in suppressed code are not reported.

use std::collections::HashSet;

use ktrs_ast::psi::{self, ImportPath, KtDotQualifiedExpression, KtImportDirective, KtPackageDirective};
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    BY_KEYWORD, CALL_EXPRESSION, DOT_QUALIFIED_EXPRESSION, FILE, IDENTIFIER, IMPORT_DIRECTIVE, KDOC_MARKDOWN_LINK,
    OPERATION_REFERENCE, PACKAGE_DIRECTIVE, REFERENCE_EXPRESSION,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::kotlin_string::{is_blank, remove_surrounding, substring_after, substring_before, substring_before_last, trim};

/// `Reference(text, inDotQualifiedExpression)`.
type Reference = (String, bool);

pub struct NoUnusedImportsRule {
    r#ref: HashSet<Reference>,
    /// A `LinkedHashSet`: its order is the emit order.
    parent_expressions: Vec<String>,
    /// A `LinkedHashMap`.
    imports: Vec<(ImportPath, NodeId)>,
    package_name: String,
    found_by_keyword: bool,
}

impl NoUnusedImportsRule {
    pub fn new() -> NoUnusedImportsRule {
        NoUnusedImportsRule {
            r#ref: HashSet::from([("*".to_owned(), false)]),
            parent_expressions: Vec::new(),
            imports: Vec::new(),
            package_name: String::new(),
            found_by_keyword: false,
        }
    }
}

impl Default for NoUnusedImportsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for NoUnusedImportsRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-unused-imports")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn is_only_when_enabled_in_editorconfig(&self) -> bool {
        true
    }

    fn ignores_ktlint_suppressions(&self) -> bool {
        true
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        match ast.element_type(node) {
            PACKAGE_DIRECTIVE => {
                let package_directive = KtPackageDirective::of(ast, node);
                self.package_name = package_directive.qualified_name(ast);
            }

            IMPORT_DIRECTIVE => {
                let import_path = KtImportDirective::of(ast, node).import_path(ast).expect("NullPointerException: importPath");
                if self.imports.iter().any(|(it, _)| *it == import_path) {
                    // Emit directly when same import occurs more than once
                    emit(ast, ast.start_offset(node), "Unused import", true).if_autocorrect_allowed(|| psi::delete(ast, node));
                } else {
                    self.imports.push((import_path, node));
                }
            }

            DOT_QUALIFIED_EXPRESSION => {
                if self.is_expression_for_static_import_with_existing_parent_import(ast, node) {
                    let parent = substring_before_last(&ast.text(node), "(").to_owned();
                    if !self.parent_expressions.contains(&parent) {
                        self.parent_expressions.push(parent);
                    }
                }
            }

            KDOC_MARKDOWN_LINK => {
                let text = ast.text(node);
                let link_text = remove_backticks_and_trim(remove_surrounding(&text, "[", "]"));
                self.r#ref.insert((substring_before(&link_text, ".").to_owned(), false));
                self.r#ref.insert((link_text.rsplit('.').next().unwrap_or_default().to_owned(), false));
            }

            REFERENCE_EXPRESSION | OPERATION_REFERENCE => {
                if !ast.is_part_of(node, IMPORT_DIRECTIVE) {
                    let identifier = if !ast.is_leaf_element(node) { ast.find_child_by_type(node, IDENTIFIER) } else { Some(node) };
                    if let Some(text) = identifier.map(|it| ast.text(it)).filter(|it| !is_blank(it)) {
                        self.r#ref.insert((remove_backticks_and_trim(&text), is_parent_dot_qualified_expression_or_null(ast, node)));
                    }
                }
            }

            BY_KEYWORD => {
                self.found_by_keyword = true;
            }

            _ => {}
        }
    }

    fn after_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != FILE {
            return;
        }
        let direct_calls: Vec<String> = self.r#ref.iter().filter(|(_, in_dot)| !in_dot).map(|(text, _)| text.clone()).collect();
        for parent in self.parent_expressions.clone() {
            let matching: Vec<(ImportPath, NodeId)> = self
                .imports
                .iter()
                .filter(|(import, _)| {
                    let import_path = remove_backticks_and_trim(&import.path_str());
                    import_path.ends_with(&format!(".{parent}")) && !direct_calls.iter().any(|it| import_path.ends_with(&format!(".{it}")))
                })
                .cloned()
                .collect();
            for (import_path, import_node) in matching {
                emit(ast, ast.start_offset(import_node), "Unused import", true).if_autocorrect_allowed(|| {
                    self.imports.retain(|(p, n)| !(*p == import_path && *n == import_node));
                    remove_import_directive(ast, import_node);
                });
            }
        }

        let referenced: HashSet<&str> = self.r#ref.iter().map(|(text, _)| text.as_str()).collect();
        for &(_, node) in &self.imports {
            let import_directive = KtImportDirective::of(ast, node);
            let path = import_directive.import_path(ast);
            let name = path.as_ref().and_then(ImportPath::imported_name).map(|it| remove_backticks_and_trim(&it));
            let import_path = remove_backticks_and_trim(&path.expect("NullPointerException: importPath").path_str());
            let package_name = &self.package_name;
            if import_directive.alias_name(ast).is_none()
                && (package_name.is_empty() || import_path.starts_with(&format!("{package_name}.")))
                && !skip_chars(&import_path, package_name.chars().count() + 1).contains('.')
            {
                // Allow imports without alias for which the fully qualified path is equal to the package name
                // (ktlint#2821: marking an import from the same package led to compile failure).
            } else if name.as_ref().is_some_and(|name| {
                (!referenced.contains(name.as_str()) || !self.is_a_valid_import(&import_path))
                    && !OPERATOR_SET.contains(&name.as_str())
                    && !is_component_n(name)
                    && !self.ignore_provide_delegate(&import_path)
            }) {
                emit(ast, ast.start_offset(node), "Unused import", true).if_autocorrect_allowed(|| {
                    match ast.next_sibling(node) {
                        None => {
                            // Last import
                            let whitespace = ast
                                .next_leaf(ast.last_child_leaf_or_self(node))
                                .filter(|&it| ast.is_white_space_with_newline(it));
                            if let Some(whitespace) = whitespace {
                                if ast.prev_leaf(node).is_none() {
                                    // Also the first import, not preceded by any text: all whitespace until the next is redundant
                                    ast.remove(whitespace);
                                } else {
                                    let text = ast.text(whitespace);
                                    let text_after_first_newline = substring_after(&text, "\n");
                                    if !is_blank(text_after_first_newline) {
                                        ast.replace_text_with(whitespace, text_after_first_newline);
                                    }
                                }
                            }
                        }
                        Some(next_sibling) => {
                            if ast.is_white_space_with_newline(next_sibling) {
                                ast.remove(next_sibling);
                            }
                        }
                    }
                    psi::delete(ast, import_directive.node());
                });
            }
        }
    }
}

impl NoUnusedImportsRule {
    fn ignore_provide_delegate(&self, import_path: &str) -> bool {
        if import_path.ends_with(".provideDelegate") {
            // Ignore provideDelegate if the `by` keyword is found anywhere in the file
            self.found_by_keyword
        } else {
            false
        }
    }

    fn is_expression_for_static_import_with_existing_parent_import(&self, ast: &Ast, node: NodeId) -> bool {
        let text = ast.text(node);
        if !contains_method_call(&text) {
            return false;
        }

        let method_call_expression = substring_before_last(&text, "(");

        // Only check static imports; identified if they start with a capital letter indicating a class name
        // rather than a sub-package
        if !method_call_expression.is_empty() && !method_call_expression.as_bytes()[0].is_ascii_uppercase() {
            return false;
        }

        let paths: Vec<String> = self.imports.iter().map(|(it, _)| remove_backticks_and_trim(&it.path_str())).collect();
        for import in paths.iter().filter(|it| it.ends_with(&format!(".{method_call_expression}"))) {
            let prefix = substring_before(import, method_call_expression);
            let count = paths.iter().filter(|it| it.starts_with(prefix)).count();
            // Parent import and static import both are present
            if count > 1 {
                return true;
            }
        }
        false
    }

    /// Whether the import being checked is present in the filtered import list.
    fn is_a_valid_import(&self, import_path: &str) -> bool {
        self.imports.iter().any(|(it, _)| remove_backticks_and_trim(&it.path_str()).contains(import_path))
    }
}

fn remove_import_directive(ast: &mut Ast, this: NodeId) {
    assert!(ast.element_type(this) == IMPORT_DIRECTIVE, "IllegalArgumentException: Failed requirement.");
    let parent = ast.parent(this);
    let neighbour = if parent.and_then(|p| ast.first_child_node(p)) == Some(this) {
        ast.next_sibling(this)
    } else if parent.and_then(|p| ast.last_child_node(p)) == Some(this) {
        ast.prev_sibling(this)
    } else {
        ast.next_leaf(this)
    };
    if let Some(whitespace) = neighbour.filter(|&it| ast.is_white_space_with_newline(it)) {
        ast.remove(whitespace);
    }
    ast.remove(this);
}

fn contains_method_call(text: &str) -> bool {
    text.rsplit('.').next().unwrap_or_default().contains('(')
}

/// `COMPONENT_N_REGEX` = `^component\d+$`.
fn is_component_n(name: &str) -> bool {
    name.strip_prefix("component").is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

fn is_parent_dot_qualified_expression_or_null(ast: &Ast, node: NodeId) -> bool {
    let call_expression_or_this = parent_call_expression_or_null(ast, node).unwrap_or(node);
    is_dot_qualified_expression(ast, call_expression_or_this)
}

fn parent_call_expression_or_null(ast: &Ast, node: NodeId) -> Option<NodeId> {
    ast.parent(node).filter(|&it| ast.element_type(it) == CALL_EXPRESSION)
}

fn is_dot_qualified_expression(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .filter(|&it| ast.element_type(it) == DOT_QUALIFIED_EXPRESSION)
        .and_then(|it| KtDotQualifiedExpression::cast(ast, it))
        .is_some_and(|it| it.selector_expression(ast) == Some(node))
}

fn remove_backticks_and_trim(s: &str) -> String {
    trim(&s.replace('`', "")).to_owned()
}

/// `substring(n)`, `n` counted in chars (UTF-16 units upstream; the same cut for the callers' prefixes).
fn skip_chars(s: &str, n: usize) -> &str {
    s.char_indices().nth(n).map_or("", |(i, _)| &s[i..])
}

const OPERATOR_SET: &[&str] = &[
    // unary
    "unaryPlus", "unaryMinus", "not",
    // inc/dec
    "inc", "dec",
    // arithmetic
    "plus", "minus", "times", "div", "rem", "mod", "rangeTo", "rangeUntil",
    // in
    "contains",
    // indexed access
    "get", "set",
    // invoke
    "invoke",
    // (augmented) assignment
    "assign", "plusAssign", "minusAssign", "timesAssign", "divAssign", "modAssign",
    // (in)equality
    "equals",
    // comparison
    "compareTo",
    // iteration (https://github.com/shyiko/ktlint/issues/40)
    "iterator",
    // by (https://github.com/shyiko/ktlint/issues/54)
    "getValue", "setValue",
];
