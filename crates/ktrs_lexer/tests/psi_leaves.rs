//! Lexer vs. the leaves of the upstream PSI fixtures (testdata/kotlin/psi). Leaf boundaries and
//! texts must match; kinds may differ only by the parser's known remappings (soft keywords, and
//! the multi-token operators the parser joins). A `KDoc` subtree counts as one `DOC_COMMENT`.

mod common;

use std::collections::BTreeMap;

use common::{convert_line_separators, spans, testdata, walk};
use ktrs_syntax::SyntaxKind;

/// Tokens the parser builds from several lexer tokens.
const JOINED: &[&str] = &["SAFE_ACCESS", "ELVIS", "EXCLEXCL"];

/// `DebugUtil.fixWhiteSpaces`.
fn escape(text: &str) -> String {
    text.replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn parse_leaf(body: &str) -> Option<(String, String)> {
    if let Some(text) = body.strip_prefix("PsiWhiteSpace('") {
        return Some(("WHITE_SPACE".into(), text.strip_suffix("')")?.into()));
    }
    let rest = body
        .strip_prefix("PsiElement(")
        .or_else(|| body.strip_prefix("PsiComment("))?;
    let (name, text) = rest.split_once(")('")?;
    Some((name.into(), text.strip_suffix("')")?.into()))
}

/// Leaves of dump `lines`, with each `node` subtree collapsed into one `leaf_kind` leaf; the
/// collapsed subtrees' lines are appended to `collapsed`.
fn expected_leaves<'d>(
    lines: &[&'d str],
    node: &str,
    leaf_kind: &str,
    collapsed: &mut Vec<Vec<&'d str>>,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut open: Option<(usize, String)> = None;
    for &line in lines {
        let body = line.trim_start();
        let indent = line.len() - body.len();
        if let Some((node_indent, text)) = &mut open {
            if indent > *node_indent {
                if let Some((_, leaf)) = parse_leaf(body) {
                    text.push_str(&leaf);
                }
                collapsed.last_mut().unwrap().push(line);
                continue;
            }
            out.push((leaf_kind.to_string(), std::mem::take(text)));
            open = None;
        }
        if body == node {
            open = Some((indent, String::new()));
            collapsed.push(Vec::new());
        } else if let Some(leaf) = parse_leaf(body) {
            out.push(leaf);
        }
    }
    if let Some((_, text)) = open {
        out.push((leaf_kind.to_string(), text));
    }
    out
}

fn leaves(spans: impl IntoIterator<Item = (SyntaxKind, impl AsRef<str>)>) -> Vec<(String, String)> {
    spans
        .into_iter()
        .filter(|(_, t)| !t.as_ref().is_empty())
        .map(|(kind, t)| (kind.debug_name().to_string(), escape(t.as_ref())))
        .collect()
}

fn compare(
    expected: &[(String, String)],
    actual: &[(String, String)],
    categories: &mut BTreeMap<String, usize>,
) -> Result<(), String> {
    let (mut i, mut j) = (0, 0);
    while i < expected.len() && j < actual.len() {
        let ((en, et), (an, at)) = (&expected[i], &actual[j]);
        let mismatch = || format!("leaf {i}: expected {en}('{et}'), lexed {an}('{at}')");
        if et == at {
            if en != an {
                let remapped = |keyword: Option<SyntaxKind>| {
                    keyword.map(SyntaxKind::debug_name) == Some(en.as_str())
                };
                // Hard keywords only as `$keyword` in a string, which `parseStringTemplateElement` remaps.
                let category = if an != "IDENTIFIER" {
                    return Err(mismatch());
                } else if remapped(SyntaxKind::soft_keyword(at)) {
                    "soft keyword"
                } else if remapped(SyntaxKind::hard_keyword(at)) {
                    "keyword in short template entry"
                } else {
                    return Err(mismatch());
                };
                *categories.entry(category.into()).or_default() += 1;
            }
            i += 1;
            j += 1;
        } else if et.starts_with(at.as_str()) && JOINED.contains(&en.as_str()) {
            let mut joined = at.clone();
            let mut k = j + 1;
            while joined.len() < et.len() && k < actual.len() {
                joined.push_str(&actual[k].1);
                k += 1;
            }
            if &joined != et {
                return Err(mismatch());
            }
            *categories.entry(format!("joined {en}")).or_default() += 1;
            i += 1;
            j = k;
        } else {
            return Err(mismatch());
        }
    }
    match (expected.get(i), actual.get(j)) {
        (None, None) => Ok(()),
        (e, a) => Err(format!(
            "leaf {i}: stream ends differ, expected {e:?}, lexed {a:?}"
        )),
    }
}

#[test]
fn psi_fixture_leaves() {
    let files = walk(&testdata("psi"), &["kt", "kts"]);
    let (mut categories, mut kdoc_categories) = (BTreeMap::new(), BTreeMap::new());
    let mut failures = Vec::new();
    let mut kdoc_count = 0;
    for source in &files {
        let Ok(dump) = std::fs::read_to_string(source.with_extension("txt")) else {
            continue;
        };
        let dump = convert_line_separators(&dump);
        let lines: Vec<&str> = dump.lines().collect();
        let text = convert_line_separators(&std::fs::read_to_string(source).unwrap());
        // The dumps were produced from the text with trailing newlines stripped.
        let text = text.trim_end_matches('\n');

        let mut kdocs = Vec::new();
        let expected = expected_leaves(
            &lines,
            "KDoc",
            SyntaxKind::DOC_COMMENT.debug_name(),
            &mut kdocs,
        );
        let tokens = spans(text, &ktrs_lexer::tokenize(text));
        if let Err(e) = compare(&expected, &leaves(tokens.iter().copied()), &mut categories) {
            failures.push(format!("{}: {e}", source.display()));
            continue;
        }
        let docs = tokens
            .iter()
            .filter(|(kind, _)| *kind == SyntaxKind::DOC_COMMENT)
            .map(|&(_, t)| t);
        for (doc, kdoc_lines) in docs.zip(&kdocs) {
            kdoc_count += 1;
            let expected = expected_leaves(
                kdoc_lines,
                "KDOC_MARKDOWN_LINK",
                "KDOC_MARKDOWN_LINK",
                &mut Vec::new(),
            );
            let actual = leaves(spans(doc, &ktrs_lexer::tokenize_kdoc(doc)));
            if let Err(e) = compare(&expected, &actual, &mut kdoc_categories) {
                failures.push(format!("{} (KDoc {doc:?}): {e}", source.display()));
            }
        }
    }
    eprintln!("{} files; accepted remappings: {categories:?}", files.len());
    eprintln!("{kdoc_count} KDoc comments; accepted remappings: {kdoc_categories:?}");
    assert!(
        files.len() >= 680,
        "only {} PSI fixtures found",
        files.len()
    );
    assert!(
        failures.is_empty(),
        "{} mismatching file(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
