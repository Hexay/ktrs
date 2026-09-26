//! Port of ktfmt's `Tokenizer.kt`: walks the parse tree (which, unlike javac's, keeps whitespace
//! and comments) and builds the list of `KotlinTok`s.
//!
//! gjf wants newline toks separate from maximal-space toks, but Kotlin emits whitespace as one
//! leaf, so it is split with `\R|( )+` (other whitespace, e.g. tabs, produces no tok).
//!
//! The walk is over the green tree: a `SyntaxNode` cursor walk allocates per element, and the
//! tokenizer only needs kinds, offsets and the two enclosing kinds of each comment.

use std::rc::Rc;

use ktrs_syntax::{SyntaxKind, SyntaxNode};
use rowan::{GreenNodeData, NodeOrToken};

use super::kotlin_tok::KotlinTok;
use super::parse_error::ParseError;
use super::whitespace_tombstones::replace_trailing_whitespace_with_tombstone;

pub struct Tokenizer<'a> {
    file_text: &'a str,
    /// `file_text`, shared by the toks.
    source: Rc<str>,
    pub toks: Vec<KotlinTok>,
    index: i32,
}

/// What a comment's `element.parent` checks need about the node being walked.
#[derive(Clone, Copy)]
struct Parent {
    kind: SyntaxKind,
    grandparent_kind: Option<SyntaxKind>,
    has_node_children: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(file_text: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            file_text,
            source: file_text.into(),
            // Roughly one tok per 3-4 bytes of typical code.
            toks: Vec::with_capacity(file_text.len() / 3),
            index: 0,
        }
    }

    pub fn index(&self) -> i32 {
        self.index
    }

    /// `file.accept(tokenizer)`.
    pub fn visit_file(&mut self, file: &SyntaxNode) -> Result<(), ParseError> {
        let start = usize::from(file.text_range().start());
        let parent = file.parent().map(|p| Parent {
            kind: p.kind(),
            grandparent_kind: p.parent().map(|g| g.kind()),
            has_node_children: true,
        });
        self.visit_element(&file.green(), start, file.kind(), parent)
    }

    /// `visitElement` for a composite, then (as `super.visitElement`) its children.
    fn visit_element(
        &mut self,
        node: &GreenNodeData,
        start: usize,
        kind: SyntaxKind,
        parent: Option<Parent>,
    ) -> Result<(), ParseError> {
        let end = start + usize::from(node.text_len());
        if !self.visit_element_self(kind, start, end, parent, false)? {
            return Ok(());
        }
        let this = Parent {
            kind,
            grandparent_kind: parent.map(|p| p.kind),
            has_node_children: node.children().any(|c| c.as_node().is_some()),
        };
        let mut offset = start;
        for child in node.children() {
            let child_kind = SyntaxKind::from_raw(child.kind().0);
            let child_end = offset + usize::from(child.text_len());
            match child {
                NodeOrToken::Node(n) => self.visit_element(n, offset, child_kind, Some(this))?,
                NodeOrToken::Token(_) => {
                    self.visit_element_self(child_kind, offset, child_end, Some(this), true)?;
                }
            }
            offset = child_end;
        }
        Ok(())
    }

    /// The body of upstream's `visitElement` before `super.visitElement`; returns whether to
    /// continue into the children.
    fn visit_element_self(
        &mut self,
        kind: SyntaxKind,
        start: usize,
        end: usize,
        parent: Option<Parent>,
        is_leaf: bool,
    ) -> Result<bool, ParseError> {
        let original_text = &self.file_text[start..end];
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
            let parent_block = parent.filter(|p| p.kind == SyntaxKind::BLOCK);
            let is_in_lambda_body =
                parent_block.is_some_and(|b| b.grandparent_kind == Some(SyntaxKind::FUNCTION_LITERAL));
            // PSI `getChildren()` of a block lists only composite children.
            let body_has_no_statements = parent_block.is_some_and(|b| !b.has_node_children);
            let treat_as_token = is_block_comment && is_in_lambda_body && body_has_no_statements;
            self.push(KotlinTok::from_source(self.index, &self.source, start..end, None, 0, treat_as_token));
            return Ok(false);
        }
        if kind == SyntaxKind::STRING_TEMPLATE {
            let text = replace_trailing_whitespace_with_tombstone(original_text);
            self.push(KotlinTok::new(self.index, text, original_text.to_string(), start as i32, 0, true));
            return Ok(false);
        }
        if is_leaf {
            if kind == SyntaxKind::WHITE_SPACE {
                for (offset, text) in split_whitespace_newlines(original_text) {
                    let range = start + offset..start + offset + text.len();
                    let tok = KotlinTok::from_source(-1, &self.source, range, None, 0, false);
                    self.toks.push(tok);
                }
            } else {
                self.push(KotlinTok::from_source(self.index, &self.source, start..end, None, 0, true));
            }
        }
        Ok(true)
    }

    /// Adds a numbered tok.
    fn push(&mut self, tok: KotlinTok) {
        self.toks.push(tok);
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
