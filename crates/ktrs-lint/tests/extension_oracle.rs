//! `ASTNodeExtension.kt` / `IndentConfig.kt` against the ktlint fat jar, node by node
//! (`tools/ktlint-oracle/ExtensionOracle.java` writes `tests/data/extension.jvm.txt`; its header has the command).
//! Columns (tab-separated, nodes named by preorder index, `-` = null, `!E` = threw E):
//! id, type, start-end (UTF-16), nextLeaf,prevLeaf, nextCodeLeaf,prevCodeLeaf, nextCodeSibling,prevCodeSibling,
//! firstChildLeafOrSelf,lastChildLeafOrSelf, flags (isCode isPartOfComment isPartOfString isWhiteSpace
//! ..WithNewline ..WithoutNewline treePrev.isWhiteSpaceWithoutNewlineOrNull isRoot isLeaf isDeclaration
//! isPartOf(COMMENTS)), column, indent|indentWithoutNewlinePrefix, leavesOnLine (first 3 # count),
//! leavesOnLine.dropTrailingEolComment().lineLength, hasNoMaxLineLengthSuppression, isKtAnnotated,
//! findChildByTypeRecursively(IDENTIFIER),recursiveChildren count, afterCodeSibling(LPAR) beforeCodeSibling(RPAR)
//! hasModifier(PRIVATE), leavesForwards|BackwardsIncludingSelf (first 2), ranges to treeNext (hasNewLineInClosed
//! noNewLineInClosed noNewLineInOpen open,closed counts), child|sibling|parentIndentOf (4 spaces), indentLevelFrom |
//! toNormalizedIndent (spaces) | toNormalizedIndent (tabs) | indentLevelFrom (tabs) | tabs' unexpected char + index.

use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

use ktrs_ast::{Ast, NodeId};
use ktrs_lint::ast_node_extension::*;
use ktrs_lint::indent_config::IndentConfig;
use ktrs_lint::rule::IndentStyle;
use ktrs_parser::kt_tokens::COMMENTS;
use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::SyntaxKind::*;

const FILES: [(&str, &str); 4] = [
    ("extension_sample.kt", include_str!("data/extension_sample.kt")),
    ("CommentsBinding.kt", include_str!("../../../testdata/kotlin/psi/CommentsBinding.kt")),
    ("FunctionLiterals.kt", include_str!("../../../testdata/kotlin/psi/FunctionLiterals.kt")),
    ("annotations.kt", include_str!("../../../testdata/kotlin/psi/annotations.kt")),
];

struct Oracle<'a> {
    ast: &'a Ast,
    index: HashMap<NodeId, usize>,
}

impl Oracle<'_> {
    fn id(&self, n: Option<NodeId>) -> String {
        n.map_or("-".to_owned(), |n| self.index[&n].to_string())
    }

    fn ids(&self, it: impl Iterator<Item = NodeId>, max: usize) -> String {
        it.take(max).map(|n| self.index[&n].to_string()).collect::<Vec<_>>().join(",")
    }

    fn line(&self, n: NodeId) -> String {
        let a = self.ast;
        let (spaces, tabs) = (IndentConfig::default_indent_config(), IndentConfig::new(IndentStyle::Tab, 4));
        let indent = a.indent(n);
        let b = |v: bool| if v { "1" } else { "0" };
        let utf16 = |byte: usize| a.utf16_offset(a.root(), byte);
        let flags = [
            a.is_code(n),
            a.is_part_of_comment(n),
            a.is_part_of_string(n),
            a.is_white_space(n),
            a.is_white_space_with_newline(n),
            a.is_white_space_without_newline(n),
            a.is_white_space_without_newline_or_null(a.tree_prev(n)),
            a.is_root(n),
            a.is_leaf(n),
            a.is_declaration(n),
            a.is_part_of_set(n, COMMENTS),
        ];
        let ranges = match a.tree_next(n) {
            None => "-".to_owned(),
            Some(next) => format!(
                "{}{}{}{},{}",
                b(a.has_new_line_in_closed_range(n, next)),
                b(a.no_new_line_in_closed_range(n, next)),
                b(a.no_new_line_in_open_range(n, next)),
                a.leaves_in_open_range(n, next).count(),
                a.leaves_in_closed_range(n, next).count()
            ),
        };
        let fields = [
            self.id(Some(n)),
            a.element_type(n).debug_name().replace("WHEN_CONDITION_EXPRESSION", "WHEN_CONDITION_WITH_EXPRESSION"),
            format!("{}-{}", utf16(a.start_offset(n)), utf16(a.end_offset(n))),
            format!("{},{}", self.id(a.next_leaf(n)), self.id(a.prev_leaf(n))),
            format!("{},{}", self.id(a.next_code_leaf(n)), self.id(a.prev_code_leaf(n))),
            format!("{},{}", self.id(a.next_code_sibling(n)), self.id(a.prev_code_sibling(n))),
            format!("{},{}", self.id(Some(a.first_child_leaf_or_self(n))), self.id(Some(a.last_child_leaf_or_self(n)))),
            flags.iter().map(|&f| b(f)).collect(),
            a.column(n).to_string(),
            format!("{}|{}", esc(&indent), esc(&a.indent_without_newline_prefix(n))),
            format!("{}#{}", self.ids(a.leaves_on_line(n), 3), a.leaves_on_line(n).count()),
            safe(|| a.line_length(a.drop_trailing_eol_comment(a.leaves_on_line(n)))),
            b(a.has_no_max_line_length_suppression(n)).to_owned(),
            safe(|| a.is_kt_annotated(n)),
            format!("{},{}", self.id(a.find_child_by_type_recursively(n, IDENTIFIER)), a.recursive_children(n).count()),
            [a.after_code_sibling(n, LPAR), a.before_code_sibling(n, RPAR), a.has_modifier(n, PRIVATE_KEYWORD)].map(b).concat(),
            format!("{}|{}", self.ids(a.leaves_forwards_including_self(n), 2), self.ids(a.leaves_backwards_including_self(n), 2)),
            ranges,
            [safe(|| spaces.child_indent_of(a, n)), safe(|| spaces.sibling_indent_of(a, n)), safe(|| spaces.parent_indent_of(a, n))]
                .join("|"),
            format!(
                "{}|{}|{}|{}|{}{}",
                safe(|| spaces.indent_level_from(&indent)),
                safe(|| spaces.to_normalized_indent(&indent)),
                safe(|| tabs.to_normalized_indent(&indent)),
                safe(|| tabs.indent_level_from(&indent)),
                tabs.contains_unexpected_indent_char(&indent),
                tabs.index_of_first_unexpected_indent_char(&indent)
            ),
        ];
        fields.join("\t")
    }
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t")
}

/// The value, or `!Exception` for a panic whose message starts with the Kotlin exception name.
fn safe<T: ToString>(f: impl FnOnce() -> T) -> String {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => esc(&v.to_string()),
        Err(e) => {
            let message = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()));
            format!("!{}", message.unwrap_or_default().split(':').next().unwrap_or(""))
        }
    }
}

#[test]
fn extensions_match_the_jvm() {
    std::panic::set_hook(Box::new(|_| {}));
    let expected = include_str!("data/extension.jvm.txt").replace("\r\n", "\n");
    let mut actual = String::new();
    for (name, text) in FILES {
        let text = text.replace("\r\n", "\n");
        let ast = Ast::from_parse(&parse_file(&text, FileKind::Source));
        let nodes: Vec<NodeId> = ast.preorder(ast.root()).collect();
        let oracle = Oracle { ast: &ast, index: nodes.iter().enumerate().map(|(i, &n)| (n, i)).collect() };
        actual.push_str(&format!("=== {name}\n"));
        for &n in &nodes {
            actual.push_str(&oracle.line(n));
            actual.push('\n');
        }
    }
    let _ = std::panic::take_hook();
    let mismatches: Vec<String> = expected
        .lines()
        .zip(actual.lines())
        .filter(|(e, a)| e != a)
        .take(8)
        .map(|(e, a)| format!("jvm:  {e}\nrust: {a}"))
        .collect();
    assert!(mismatches.is_empty() && expected.lines().count() == actual.lines().count(), "{}", mismatches.join("\n"));
}
