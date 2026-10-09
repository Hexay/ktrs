//! `detekt-metrics/.../LinesOfCode.kt`.

use ktrs_psi::PsiElement;
use ktrs_syntax::SyntaxKind::{self, *};

use crate::kt_file;

/// `KtElement.linesOfCode()`: the number of distinct lines a token of the element starts on, without the
/// subtrees of the `comments` classes.
///
/// Upstream walks `tokenSequence` breadth-first and collects the lines in a set. The count does not depend on
/// the order, and in preorder the tokens' offsets only grow, so a token starts a new line exactly when it lies
/// past the end of the last counted one.
pub fn lines_of_code(element: &PsiElement) -> usize {
    let file = kt_file::containing_file(element);
    let tree = element.tree();
    let end = tree.subtree_end(element.id());
    let mut id = element.id();
    let mut count = 0;
    let mut counted_line_end = 0;
    while id < end {
        let is_token = tree.is_token(id);
        if skips_tree(tree.kind(id), is_token) {
            id = tree.subtree_end(id);
            continue;
        }
        if is_token {
            let start = usize::from(tree.text_range(id).start());
            if count == 0 || start >= counted_line_end {
                count += 1;
                counted_line_end = file.next_line_start_offset(file.line_number(start));
            }
        }
        id += 1;
    }
    count
}

/// `curr.psi::class.java in comments`. Gotcha: the test is on the exact class, so `KDocImpl` is not skipped and
/// its `/**` and `*/` tokens count; its sections are.
fn skips_tree(kind: SyntaxKind, is_token: bool) -> bool {
    if is_token {
        matches!(kind, WHITE_SPACE | EOL_COMMENT | BLOCK_COMMENT | SHEBANG_COMMENT)
    } else {
        matches!(kind, KDOC_SECTION | KDOC_TAG | KDOC_NAME | KDOC_MARKDOWN_LINK)
    }
}
