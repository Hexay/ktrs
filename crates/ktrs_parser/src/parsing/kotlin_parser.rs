//! Port of `KotlinParser.java` plus the chameleon (`ILazyParseableElementType.parseContents`)
//! reparse used when building trees.

use ktrs_syntax::{Parse, SyntaxKind};

use super::Parser;
use crate::FileKind;
use crate::builder::{LazyLeaf, PsiBuilder, SemanticWhitespaceAwarePsiBuilder, TreeSink};

/// `KotlinParser.parse(psiBuilder, psiFile)`: `parseFile` or `parseScript` by file kind.
pub fn parse(text: &str, kind: FileKind) -> Parse {
    run_top_level(text, |kt_parsing| match kind {
        FileKind::Source => kt_parsing.parse_file(),
        FileKind::Script => kt_parsing.parse_script(),
    })
}

pub fn parse_type_code_fragment(text: &str) -> Parse {
    run_top_level(text, Parser::parse_type_code_fragment)
}

pub fn parse_expression_code_fragment(text: &str) -> Parse {
    run_top_level(text, Parser::parse_expression_code_fragment)
}

pub fn parse_block_code_fragment(text: &str) -> Parse {
    run_top_level(text, Parser::parse_block_code_fragment)
}

/// `LambdaExpressionElementType.parseContents`: the reparse of a collapsed `LAMBDA_EXPRESSION`.
pub fn parse_lambda_expression(text: &str) -> Parse {
    run_top_level(text, Parser::parse_lambda_expression)
}

/// `BlockExpressionElementType.parseContents`: the reparse of a collapsed `BLOCK`.
pub fn parse_block_expression(text: &str) -> Parse {
    run_top_level(text, Parser::parse_block_expression)
}

/// `KotlinParsing.createForTopLevel(new SemanticWhitespaceAwarePsiBuilderImpl(psiBuilder))`, run,
/// then `psiBuilder.getTreeBuilt()`.
fn run_top_level(text: &str, parse: impl FnOnce(&mut Parser)) -> Parse {
    let mut sink = TreeSink::new();
    run_into(PsiBuilder::lex_kotlin(text), parse, None, &mut sink);
    sink.finish()
}

fn run_into(psi: PsiBuilder, parse: impl FnOnce(&mut Parser), root_kind: Option<SyntaxKind>, sink: &mut TreeSink) {
    let mut kt_parsing = Parser::create_for_top_level(SemanticWhitespaceAwarePsiBuilder::new(psi));
    parse(&mut kt_parsing);
    kt_parsing.my_builder.psi.build_tree_into(root_kind, sink, &reparse_lazy);
}

/// Expands a lazy-parseable leaf: its kind's `parseContents` over its own text. The reparsed
/// root's children become the leaf node's children (`getFirstChildNode()` upstream).
/// Kotlin's lazy types don't `reuseCollapsedTokens`, so remaps made by the outer parse are
/// forgotten; the leaf's unremapped lexemes stand in for re-lexing its text (see `LazyLeaf`).
pub(crate) fn reparse_lazy(leaf: &LazyLeaf<'_>, sink: &mut TreeSink) -> bool {
    let parse: fn(&mut Parser) = match leaf.kind {
        SyntaxKind::BLOCK => Parser::parse_block_expression,
        SyntaxKind::LAMBDA_EXPRESSION => Parser::parse_lambda_expression,
        SyntaxKind::DOC_COMMENT => {
            crate::kdoc::parse_kdoc_into(leaf.text, sink);
            return true;
        }
        SyntaxKind::KDOC_MARKDOWN_LINK => {
            crate::kdoc::parse_markdown_link_into(leaf.text, sink);
            return true;
        }
        _ => return false,
    };
    run_into(leaf.relexed_builder(), parse, Some(leaf.kind), sink);
    true
}
