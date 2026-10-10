use std::fmt::Write;

use crate::{Node, SourceFile, SyntaxKind, WalkEvent};

impl SourceFile {
    /// The tree in the format of the Kotlin compiler's `DebugUtil.psiToString`, which is what its parser tests
    /// compare against: one line per node and token, two spaces of indentation per level.
    ///
    /// ```
    /// let file = kt_syntax::parse("val x").unwrap();
    /// let dump = file.dump();
    /// assert!(dump.starts_with("KtFile: \n  PACKAGE_DIRECTIVE\n    <empty list>\n  IMPORT_LIST\n"));
    /// assert!(dump.ends_with("  PROPERTY\n    PsiElement(val)('val')\n    PsiWhiteSpace(' ')\n    PsiElement(IDENTIFIER)('x')"));
    /// ```
    pub fn dump(&self) -> String {
        self.dump_named("")
    }

    /// [`dump`](SourceFile::dump) with a file name in the first line, as the compiler prints it.
    pub fn dump_named(&self, file_name: &str) -> String {
        let mut out = format!("KtFile: {file_name}\n");
        let mut depth = 0;
        for event in self.root().preorder() {
            match event {
                WalkEvent::Enter(node) => {
                    if depth > 0 {
                        line(&mut out, depth, node);
                    }
                    depth += 1;
                    if !node.is_token() && node.first_child().is_none() {
                        indent(&mut out, depth);
                        out.push_str("<empty list>\n");
                    }
                }
                WalkEvent::Leave(_) => depth -= 1,
            }
        }
        out.truncate(out.trim_end().len());
        out
    }
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn line(out: &mut String, depth: usize, node: Node<'_>) {
    indent(out, depth);
    let kind = node.kind();
    if !node.is_token() {
        match node.error_message() {
            Some(message) => writeln!(out, "PsiErrorElement:{message}"),
            None => writeln!(out, "{}", kind.dump_name()),
        }
        .unwrap();
        return;
    }
    // `DebugUtil.fixWhiteSpaces`.
    let text = node.text().replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t");
    match kind {
        SyntaxKind::WHITE_SPACE => writeln!(out, "PsiWhiteSpace('{text}')"),
        SyntaxKind::EOL_COMMENT | SyntaxKind::BLOCK_COMMENT | SyntaxKind::SHEBANG_COMMENT => {
            writeln!(out, "PsiComment({})('{text}')", kind.dump_name())
        }
        _ => writeln!(out, "PsiElement({})('{text}')", kind.dump_name()),
    }
    .unwrap();
}
