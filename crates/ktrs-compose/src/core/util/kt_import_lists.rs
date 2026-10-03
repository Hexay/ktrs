//! Port of `core/util/KtImportLists.kt`.

use ktrs_ast::psi::KtImportList;
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::*;

/// `KtImportList.sort()`: the import directives sorted and deduplicated by text, one per line, replacing every
/// child of the list.
pub fn sort(ast: &mut Ast, import_list: KtImportList) {
    let node = import_list.node();
    let mut sorted_imports: Vec<NodeId> = ast.children(node).filter(|&c| ast.element_type(c) == IMPORT_DIRECTIVE).collect();
    sorted_imports.sort_by_key(|&it| ast.text(it));
    let mut seen: Vec<String> = Vec::new();
    sorted_imports.retain(|&it| {
        let text = ast.text(it);
        let first = !seen.contains(&text);
        seen.push(text);
        first
    });
    if let Some(first) = ast.first_child_node(node) {
        ast.remove_range(node, first, None);
    }
    for (index, ast_node) in sorted_imports.into_iter().enumerate() {
        if index > 0 {
            let whitespace = ast.new_leaf(WHITE_SPACE, "\n");
            ast.add_child(node, whitespace, None);
        }
        ast.add_child(node, ast_node, None);
    }
}

/// `KtImportList.addImports(vararg imports)`: appends `import a.b.C` directives built leaf by leaf, then [`sort`]s.
pub fn add_imports(ast: &mut Ast, import_list: KtImportList, imports: &[&str]) {
    for import in imports {
        let new_import = ast.new_composite(IMPORT_DIRECTIVE);
        let mut leaves = vec![ast.new_leaf(IMPORT_KEYWORD, "import"), ast.new_leaf(WHITE_SPACE, " ")];
        for (index, s) in import.split('.').enumerate() {
            if index != 0 {
                leaves.push(ast.new_leaf(DOT, "."));
            }
            leaves.push(ast.new_leaf(IDENTIFIER, s));
        }
        for leaf in leaves {
            ast.raw_add_children(new_import, leaf);
        }
        ast.add_child(import_list.node(), new_import, None);
    }
    sort(ast, import_list);
}
