use std::fmt::Write;

use ktrs_syntax::SyntaxKind;

use crate::arena::{Ast, NodeId};

impl Ast {
    /// `DebugUtil.psiToString(file, true, false)` of the (possibly mutated) tree under `root`, identical
    /// to `ktrs_syntax::psi_dump` for an unedited tree. The PSI class of each leaf follows from its type
    /// (`PsiWhiteSpaceImpl`, `PsiCommentImpl`, else `LeafPsiElement`), as for every leaf ktlint builds.
    pub fn psi_to_string(&self, root: NodeId, file_name: &str) -> String {
        let mut out = String::with_capacity(self.text_length(root) * 8);
        writeln!(out, "KtFile: {file_name}").unwrap();
        self.children_to_buffer(&mut out, root, 1);
        out.truncate(out.trim_end().len());
        out
    }

    fn children_to_buffer(&self, out: &mut String, e: NodeId, depth: usize) {
        if self.first_child_node(e).is_none() {
            indent(out, depth);
            out.push_str("<empty list>\n");
        }
        let mut child = self.first_child_node(e);
        while let Some(c) = child {
            indent(out, depth);
            let kind = self.element_type(c);
            if self.is_leaf_element(c) {
                let text = self.leaf_text(c).replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t");
                match kind {
                    SyntaxKind::WHITE_SPACE => write!(out, "PsiWhiteSpace('{text}')"),
                    SyntaxKind::EOL_COMMENT | SyntaxKind::BLOCK_COMMENT | SyntaxKind::SHEBANG_COMMENT => {
                        write!(out, "PsiComment({})('{text}')", kind.debug_name())
                    }
                    _ => write!(out, "PsiElement({})('{text}')", kind.debug_name()),
                }
                .unwrap();
                out.push('\n');
            } else {
                if kind == SyntaxKind::ERROR_ELEMENT {
                    write!(out, "PsiErrorElement:{}", self.error_description(c)).unwrap();
                } else {
                    out.push_str(kind.debug_name());
                }
                out.push('\n');
                self.children_to_buffer(out, c, depth + 1);
            }
            child = self.tree_next(c);
        }
    }
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}
