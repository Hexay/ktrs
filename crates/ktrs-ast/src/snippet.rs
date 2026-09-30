use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::SyntaxKind;

use crate::arena::{Ast, NodeId};

impl Ast {
    /// ktlint's `KtlintKotlinCompiler.createASTNodeFromText(text)`: parses `text` as `File.kts` into a
    /// new file element of this arena and returns its `SCRIPT > BLOCK > SCRIPT_INITIALIZER` (or the
    /// `BLOCK` when there is none). The node stays inside its own file element, as on the JVM.
    pub fn create_ast_node_from_text(&mut self, text: &str) -> Option<NodeId> {
        let parse = parse_file(text, FileKind::Script);
        let file = self.seed(&parse);
        let block = self.find_child_by_type(file, SyntaxKind::SCRIPT).and_then(|s| self.find_child_by_type(s, SyntaxKind::BLOCK))?;
        Some(self.find_child_by_type(block, SyntaxKind::SCRIPT_INITIALIZER).unwrap_or(block))
    }
}
