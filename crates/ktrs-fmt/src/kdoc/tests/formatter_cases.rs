//! `KDocFormatterTest.kt`: one test per upstream `@Test`, in upstream order, replaying the
//! `checkFormatter` calls recorded in `data/<name>.txt`.

use super::harness::run_cases;

macro_rules! cases {
    ($($name:ident),* $(,)?) => {$(
        #[test]
        #[allow(non_snake_case)]
        fn $name() {
            run_cases(stringify!($name), include_str!(concat!("data/", stringify!($name), ".txt")));
        }
    )*};
}

cases!(
    test1, testWithOffset, testWordBreaking, testHeader, testSingle, testEmpty, testJavadocParams,
    testBracketParam, testMultiLineLink, testPreformattedWithinCode, testPreStability,
    testPreStability2, testConvertParamReference, testLineWidth1, testBlockTagsNoSeparators,
    testBlockTagsHangingIndents, testGreedyBlockIndent, testBlockTagsHangingIndents2,
    testSingleLine, testPunctuationWithLabelLink, testWrappingOfLinkText,
    testPreformattedTextIndented, testPreformattedText, testPreformattedText2,
    testPreformattedText3, testPreformattedTextWithBlankLines,
    testPreformattedTextWithBlankLinesAndTrailingSpaces, testPreformattedTextSeparation,
    testSeparateParagraphMarkers1, testSeparateParagraphMarkers2, testConvertMarkup,
    testFormattingList, testList1, testIndentedList, testDocTags, testAtInMiddle,
    testMaxCommentWidth, testHorizontalRuler, testQuoteOnlyOnFirstLine, testNoBreakUrl,
    testMaxCommentWidthDefaultsToMaxLineWidth, testAsciiArt, testAsciiArt2, testAsciiArt3,
    testBrokenAsciiArt, testHtmlLists, testVariousMarkup, testLineComments, testMoreLineComments,
    testListContinuations, testListContinuations2, testAccidentalHeader, testTODO, testReorderTags,
    testNoReorderSample, testKDocOrdering, testHtml, testPreserveParagraph, testWordJoining,
    testEarlyBreakForTodo, testPreformat, testConvertLinks, testNestedBullets,
    testTripledQuotedPrefixNotBreakable, testGreedyLineBreak, test193246766, test203584301,
    test209435082, test236743270, test238279769, testPropertiesAreParams, testKnit, testNPE,
    testExtraNewlines, testQuotedBug, testListBreaking, testNewList, testSplashScreen,
    testRaggedIndentation, testCustomKDocTag, testTables, testTableMixedWithHtml,
    testTableExtraCells, testTables2, testTables3, testTables4, testTablesEmptyCells, testTables5,
    testTables6, testTables7, testTables7b, testBulletsUnderParamTags, testLineBreaking,
    testPreTag, testPreTag2, testPreTag3, testNoConversionInReferences, testCaseSensitiveMarkup,
    testAsteriskRemoval, testParagraphTagRemoval, testDashedLineIndentation, testParagraphRemoval,
    testParagraphRemoval2, testAtBreak2, testNoBreakAfterAt, testPreCodeConversion,
    testPreConversion2, testOpenRange, testPropertiesWithBrackets, testHandingIndent,
    testMarkupAcrossLines, testReferences, testDecapitalizeKdocTags, testLineBreak,
    testDocTagsInsidePreformatted, testConvertMarkup2, testBlankLineBeforeBlockQuote,
    testBlankLineBetweenLists, testBlankLinesInMarkdownElements, testFencedCodeBlockInListItem,
    testFencedCodeBlockWithContinuationInListItem, testMultipleFencedCodeBlocksInListItem,
    testFencedCodeBlockAtEndOfListItem, testFencedCodeBlockInNumberedListItem,
    testNestedWithinQuoted,
);
