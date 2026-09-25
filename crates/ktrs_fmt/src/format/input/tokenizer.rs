//! Port of ktfmt's `Tokenizer.kt`: walks the parse tree (which, unlike javac's, keeps whitespace
//! and comments) and builds the list of `KotlinTok`s.
//!
//! gjf wants newline toks separate from maximal-space toks, but Kotlin emits whitespace as one
//! leaf, so it is split with `\R|( )+` (other whitespace, e.g. tabs, produces no tok).

use ktrs_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

use super::kotlin_tok::KotlinTok;
use super::parse_error::ParseError;
use super::whitespace_tombstones::replace_trailing_whitespace_with_tombstone;

pub struct Tokenizer<'a> {
    file_text: &'a str,
    pub toks: Vec<KotlinTok>,
    index: i32,
}

impl<'a> Tokenizer<'a> {
    pub fn new(file_text: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            file_text,
            toks: Vec::new(),
            index: 0,
        }
    }

    pub fn index(&self) -> i32 {
        self.index
    }

    /// `file.accept(tokenizer)`.
    pub fn visit_file(&mut self, file: &SyntaxNode) -> Result<(), ParseError> {
        self.visit_element(&SyntaxElement::from(file.clone()))
    }

    pub fn visit_element(&mut self, element: &SyntaxElement) -> Result<(), ParseError> {
        if self.visit_element_self(element)? {
            // super.visitElement: visit the children.
            if let Some(node) = element.as_node() {
                for child in node.children_with_tokens() {
                    self.visit_element(&child)?;
                }
            }
        }
        Ok(())
    }

    /// The body of upstream's `visitElement` before `super.visitElement`; returns whether to
    /// continue into the children.
    fn visit_element_self(&mut self, element: &SyntaxElement) -> Result<bool, ParseError> {
        let range = element.text_range();
        let start = usize::from(range.start());
        let original_text = &self.file_text[start..usize::from(range.end())];
        let kind = element.kind();
        if is_psi_comment(kind) {
            // For a leaf or KDoc, `element.text` is the source text.
            let element_text = original_text;
            if element_text.starts_with("/*") && !element_text.ends_with("*/") {
                return Err(ParseError::at_offset(
                    "Unclosed comment",
                    self.file_text,
                    start,
                ));
            }
            // Block comments inside statement-less lambda bodies are tokens, so the visitor can
            // position them with proper break structure.
            let is_block_comment = element_text.starts_with("/*");
            let parent_block = element.parent().filter(|p| p.kind() == SyntaxKind::BLOCK);
            let is_in_lambda_body = parent_block
                .as_ref()
                .and_then(SyntaxNode::parent)
                .is_some_and(|p| p.kind() == SyntaxKind::FUNCTION_LITERAL);
            // PSI `getChildren()` of a block lists only composite children.
            let body_has_no_statements = parent_block
                .as_ref()
                .is_some_and(|b| b.children().next().is_none());
            let treat_as_token = is_block_comment && is_in_lambda_body && body_has_no_statements;
            self.push(
                original_text.to_string(),
                element_text.to_string(),
                start,
                treat_as_token,
            );
            return Ok(false);
        }
        if kind == SyntaxKind::STRING_TEMPLATE {
            let text = replace_trailing_whitespace_with_tombstone(original_text);
            self.push(text, original_text.to_string(), start, true);
            return Ok(false);
        }
        if element.as_token().is_some() {
            if kind == SyntaxKind::WHITE_SPACE {
                for (offset, text) in split_whitespace_newlines(original_text) {
                    let tok = KotlinTok::new(
                        -1,
                        text.to_string(),
                        text.to_string(),
                        (start + offset) as i32,
                        0,
                        false,
                    );
                    self.toks.push(tok);
                }
            } else {
                self.push(
                    original_text.to_string(),
                    original_text.to_string(),
                    start,
                    true,
                );
            }
        }
        Ok(true)
    }

    fn push(&mut self, original_text: String, text: String, position: usize, is_token: bool) {
        self.toks.push(KotlinTok::new(
            self.index,
            original_text,
            text,
            position as i32,
            0,
            is_token,
        ));
        self.index += 1;
    }
}

/// Element types whose PSI is a `PsiComment` (`KDoc` is a composite, the rest are leaves).
fn is_psi_comment(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::EOL_COMMENT
            | SyntaxKind::BLOCK_COMMENT
            | SyntaxKind::SHEBANG_COMMENT
            | SyntaxKind::DOC_COMMENT
    )
}

/// Matches of Java's `\R|( )+`, as `(byte offset, text)`.
fn split_whitespace_newlines(text: &str) -> Vec<(usize, &str)> {
    let mut result = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < text.len() {
        let c = text[i..].chars().next().unwrap();
        let len = if text[i..].starts_with("\r\n") {
            2
        } else if matches!(
            c,
            '\n' | '\u{0b}' | '\u{0c}' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}'
        ) {
            c.len_utf8()
        } else if c == ' ' {
            bytes[i..].iter().take_while(|&&b| b == b' ').count()
        } else {
            i += c.len_utf8();
            continue;
        };
        result.push((i, &text[i..i + len]));
        i += len;
    }
    result
}
