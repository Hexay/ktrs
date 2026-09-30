use std::fmt::Write;

use crate::{ElementId, Parse, SyntaxKind, Tree};

/// Renders `parse` exactly like IntelliJ's `DebugUtil.psiToString(file, true, false)`,
/// the format of the compiler's parser fixtures and of `tools/psi-dump`.
pub fn psi_dump(parse: &Parse, file_name: &str) -> String {
    let mut printer = Printer::new(parse, file_name);
    printer.children(&parse.tree, Tree::ROOT, 1);
    printer.finish()
}

struct Printer<'a> {
    out: String,
    errors: std::slice::Iter<'a, String>,
}

impl<'a> Printer<'a> {
    fn new(parse: &'a Parse, file_name: &str) -> Printer<'a> {
        let mut out = String::new();
        writeln!(out, "KtFile: {file_name}").unwrap();
        Printer { out, errors: parse.error_messages.iter() }
    }

    fn finish(mut self) -> String {
        self.out.truncate(self.out.trim_end().len());
        self.out
    }

    fn children(&mut self, tree: &Tree, e: ElementId, depth: usize) {
        let mut any = false;
        for child in tree.children(e) {
            any = true;
            if tree.is_token(child) {
                self.token(tree.kind(child), tree.text_of(child), depth);
            } else {
                self.node(tree.kind(child), depth);
                self.children(tree, child, depth + 1);
            }
        }
        if !any {
            self.line(depth, format_args!("<empty list>"));
        }
    }

    fn token(&mut self, kind: SyntaxKind, text: &str, depth: usize) {
        let text = escape(text);
        match kind {
            SyntaxKind::WHITE_SPACE => self.line(depth, format_args!("PsiWhiteSpace('{text}')")),
            SyntaxKind::EOL_COMMENT | SyntaxKind::BLOCK_COMMENT | SyntaxKind::SHEBANG_COMMENT => {
                self.line(depth, format_args!("PsiComment({})('{text}')", kind.debug_name()))
            }
            _ => self.line(depth, format_args!("PsiElement({})('{text}')", kind.debug_name())),
        }
    }

    fn node(&mut self, kind: SyntaxKind, depth: usize) {
        if kind == SyntaxKind::ERROR_ELEMENT {
            let message = self.errors.next().map_or("", String::as_str);
            self.line(depth, format_args!("PsiErrorElement:{message}"));
        } else {
            self.line(depth, format_args!("{}", kind.debug_name()));
        }
    }

    fn line(&mut self, depth: usize, content: std::fmt::Arguments) {
        for _ in 0..depth {
            self.out.push_str("  ");
        }
        self.out.write_fmt(content).unwrap();
        self.out.push('\n');
    }
}

/// `DebugUtil.fixWhiteSpaces`.
fn escape(text: &str) -> String {
    text.replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")
}
