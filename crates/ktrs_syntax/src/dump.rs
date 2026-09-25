use std::fmt::Write;

use crate::{Parse, SyntaxElement, SyntaxKind, SyntaxNode};

/// Renders `parse` exactly like IntelliJ's `DebugUtil.psiToString(file, true, false)`,
/// the format of the compiler's parser fixtures and of `tools/psi-dump`.
pub fn psi_dump(parse: &Parse, file_name: &str) -> String {
    let mut printer = Printer { out: String::new(), errors: parse.error_messages.iter() };
    writeln!(printer.out, "KtFile: {file_name}").unwrap();
    printer.children(&parse.syntax(), 1);
    printer.out.truncate(printer.out.trim_end().len());
    printer.out
}

struct Printer<'a> {
    out: String,
    errors: std::slice::Iter<'a, String>,
}

impl Printer<'_> {
    fn children(&mut self, node: &SyntaxNode, depth: usize) {
        let mut any = false;
        for child in node.children_with_tokens() {
            any = true;
            self.element(&child, depth);
        }
        if !any {
            self.line(depth, format_args!("<empty list>"));
        }
    }

    fn element(&mut self, element: &SyntaxElement, depth: usize) {
        match element {
            SyntaxElement::Token(token) => {
                let text = escape(token.text());
                let kind = token.kind();
                match kind {
                    SyntaxKind::WHITE_SPACE => self.line(depth, format_args!("PsiWhiteSpace('{text}')")),
                    SyntaxKind::EOL_COMMENT | SyntaxKind::BLOCK_COMMENT | SyntaxKind::SHEBANG_COMMENT => {
                        self.line(depth, format_args!("PsiComment({})('{text}')", kind.debug_name()))
                    }
                    _ => self.line(depth, format_args!("PsiElement({})('{text}')", kind.debug_name())),
                }
            }
            SyntaxElement::Node(node) => {
                if node.kind() == SyntaxKind::ERROR_ELEMENT {
                    let message = self.errors.next().map_or("", String::as_str);
                    self.line(depth, format_args!("PsiErrorElement:{message}"));
                } else {
                    self.line(depth, format_args!("{}", node.kind().debug_name()));
                }
                self.children(node, depth + 1);
            }
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
