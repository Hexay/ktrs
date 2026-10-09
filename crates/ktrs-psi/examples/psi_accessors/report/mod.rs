//! Rust side of the differential PSI report; byte-for-byte the format of
//! tools/psi-accessors/src/PsiAccessors.java (offsets converted to UTF-16 like the JVM's).

mod classes;
mod control;
mod decls;
mod exprs;
mod members;
mod recorder;

use std::path::{Path, PathBuf};

use ktrs_parser::{FileKind, parse_file};
use ktrs_psi::*;
use ktrs_syntax::Parse;

pub struct Ctx<'a> {
    out: String,
    utf16: Vec<usize>,
    parse: &'a Parse,
}

impl Ctx<'_> {
    fn off(&self, byte: usize) -> usize {
        self.utf16[byte]
    }

    pub fn ref_of(&self, e: &PsiElement) -> String {
        let kind = if e.is_file() { "FILE" } else { e.kind().debug_name() };
        format!("{kind}@{}..{}", self.off(e.start_offset()), self.off(e.end_offset()))
    }

    /// Nullable accessor.
    pub fn opt<T: PsiType>(&self, x: Option<T>) -> String {
        x.map_or_else(|| "null".to_owned(), |x| self.ref_of(x.psi()))
    }

    /// Accessor that throws upstream where we return None.
    pub fn req<T: PsiType>(&self, x: Option<T>) -> String {
        x.map_or_else(|| "!".to_owned(), |x| self.ref_of(x.psi()))
    }

    pub fn list<T: PsiType>(&self, xs: Vec<T>) -> String {
        let refs: Vec<String> = xs.iter().map(|x| self.ref_of(x.psi())).collect();
        format!("[{}]", refs.join(", "))
    }

    pub fn line(&mut self, key: &str, value: String) {
        self.out.push(' ');
        self.out.push_str(key);
        self.out.push('=');
        self.out.push_str(&value);
        self.out.push('\n');
    }
}

pub fn str_value(s: Option<&str>) -> String {
    match s {
        None => "null".to_owned(),
        Some(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('\n', "\\n").replace('\r', "\\r")),
    }
}

/// Reads like the JVM oracle: CRLF -> LF, and for `fixture` trailing newlines stripped.
pub fn normalize(raw: &str, fixture: bool) -> String {
    let text = raw.replace("\r\n", "\n");
    if fixture { text.trim_end_matches('\n').to_owned() } else { text }
}

pub fn report(text: &str, file_name: &str, script: bool) -> String {
    let kind = if script { FileKind::Script } else { FileKind::from_file_name(file_name) };
    let parse = parse_file(text, kind);
    let mut utf16 = Vec::with_capacity(text.len() + 1);
    let mut units = 0;
    for c in text.chars() {
        for _ in 0..c.len_utf8() {
            utf16.push(units);
        }
        units += c.len_utf16();
    }
    utf16.push(units);
    let mut ctx = Ctx { out: String::new(), utf16, parse: &parse };
    let file = KtFile::new(&parse);
    let mut stack = vec![PsiElement::from(file.clone())];
    while let Some(e) = stack.pop() {
        describe(&mut ctx, &e);
        let children: Vec<PsiElement> = e.all_children().collect();
        stack.extend(children.into_iter().rev());
    }
    ctx.out.push_str("walk\n");
    let walk = recorder::walk(&ctx, &file);
    ctx.out.push_str(&walk);
    ctx.out
}

type TypeTest = (&'static str, fn(&PsiElement) -> bool);

const TYPES: &[TypeTest] = &[
    ("PsiComment", PsiElement::is::<PsiComment>),
    ("PsiWhiteSpace", PsiElement::is::<PsiWhiteSpace>),
    ("LeafPsiElement", PsiElement::is::<LeafPsiElement>),
    ("PsiErrorElement", PsiElement::is::<PsiErrorElement>),
    ("KtElement", PsiElement::is::<KtElement>),
    ("KtExpression", PsiElement::is::<KtExpression>),
    ("KtDeclaration", PsiElement::is::<KtDeclaration>),
    ("KtNamedDeclaration", PsiElement::is::<KtNamedDeclaration>),
    ("KtCallableDeclaration", PsiElement::is::<KtCallableDeclaration>),
    ("KtDeclarationWithBody", PsiElement::is::<KtDeclarationWithBody>),
    ("KtFunction", PsiElement::is::<KtFunction>),
    ("KtClassOrObject", PsiElement::is::<KtClassOrObject>),
    ("KtClass", PsiElement::is::<KtClass>),
    ("KtModifierListOwner", PsiElement::is::<KtModifierListOwner>),
    ("KtTypeParameterListOwner", PsiElement::is::<KtTypeParameterListOwner>),
    ("KtValVarKeywordOwner", PsiElement::is::<KtValVarKeywordOwner>),
    ("KtDeclarationWithInitializer", PsiElement::is::<KtDeclarationWithInitializer>),
    ("KtConstructor", PsiElement::is::<KtConstructor>),
    ("KtAnonymousInitializer", PsiElement::is::<KtAnonymousInitializer>),
    ("KtQualifiedExpression", PsiElement::is::<KtQualifiedExpression>),
    ("KtReferenceExpression", PsiElement::is::<KtReferenceExpression>),
    ("KtSimpleNameExpression", PsiElement::is::<KtSimpleNameExpression>),
    ("KtCallElement", PsiElement::is::<KtCallElement>),
    ("KtUnaryExpression", PsiElement::is::<KtUnaryExpression>),
    ("KtExpressionWithLabel", PsiElement::is::<KtExpressionWithLabel>),
    ("KtLoopExpression", PsiElement::is::<KtLoopExpression>),
    ("KtWhileExpressionBase", PsiElement::is::<KtWhileExpressionBase>),
    ("KtDoubleColonExpression", PsiElement::is::<KtDoubleColonExpression>),
    ("KtContainerNode", PsiElement::is::<KtContainerNode>),
    ("KtContainerNodeForControlStructureBody", PsiElement::is::<KtContainerNodeForControlStructureBody>),
    ("KtValueArgument", PsiElement::is::<KtValueArgument>),
    ("KtTypeElement", PsiElement::is::<KtTypeElement>),
    ("KtWhenCondition", PsiElement::is::<KtWhenCondition>),
    ("KtStringTemplateEntry", PsiElement::is::<KtStringTemplateEntry>),
    ("KtStringTemplateEntryWithExpression", PsiElement::is::<KtStringTemplateEntryWithExpression>),
    ("KtSuperTypeListEntry", PsiElement::is::<KtSuperTypeListEntry>),
    ("KtModifierList", PsiElement::is::<KtModifierList>),
    ("KtContextParameterList", PsiElement::is::<KtContextParameterList>),
    ("KtContextReceiverList", PsiElement::is::<KtContextReceiverList>),
    ("KDoc", PsiElement::is::<KDoc>),
    ("KDocTag", PsiElement::is::<KDocTag>),
];

fn describe(ctx: &mut Ctx, e: &PsiElement) {
    let header = format!("{} {}\n", ctx.ref_of(e), psi_class_name(e));
    ctx.out.push_str(&header);
    let is: Vec<&str> = TYPES.iter().filter(|(_, test)| test(e)).map(|(name, _)| *name).collect();
    ctx.out.push_str(&format!(" is={}\n", is.join(",")));
    let visit = recorder::dispatch_chain(ctx, e);
    ctx.out.push_str(&format!(" visit={visit}\n"));

    ctx.line("children", ctx.list(e.children()));
    ctx.line("parent", ctx.opt(e.parent()));
    ctx.line("firstChild", ctx.opt(e.first_child()));
    ctx.line("lastChild", ctx.opt(e.last_child()));
    ctx.line("nextSibling", ctx.opt(e.next_sibling()));
    ctx.line("prevSibling", ctx.opt(e.prev_sibling()));
    ctx.line("nodeIsPsi", e.node().is_psi_element().to_string());
    ctx.line("startsWithComment", e.starts_with_comment().to_string());
    ctx.line("prevSiblingIgnoringWhitespace", ctx.opt(e.get_prev_sibling_ignoring_whitespace(false)));
    ctx.line("nextSiblingIgnoringWhitespace", ctx.opt(e.get_next_sibling_ignoring_whitespace(false)));
    ctx.line("prevSiblingIgnoringWhitespaceAndComments", ctx.opt(e.get_prev_sibling_ignoring_whitespace_and_comments(false)));
    ctx.line(
        "prevSiblingIgnoringWhitespaceAndComments(withItself)",
        ctx.opt(e.get_prev_sibling_ignoring_whitespace_and_comments(true)),
    );
    ctx.line("nextSiblingIgnoringWhitespaceAndComments", ctx.opt(e.get_next_sibling_ignoring_whitespace_and_comments(false)));
    ctx.line("prevLeaf", ctx.opt(e.prev_leaf(false)));
    ctx.line("parentOfType<KtStringTemplateExpression>", ctx.opt(e.get_parent_of_type::<KtStringTemplateExpression>(false)));

    decls::describe(ctx, e);
    exprs::describe(ctx, e);
    members::describe(ctx, e);
}

/// `.kt`/`.kts` files under `dir` as (relative path with `/`, path); `fixture` keeps only those with a
/// sibling `.txt` (the compiler's parser fixtures).
pub fn files(dir: &Path, fixture: bool) -> Vec<(String, PathBuf)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if matches!(path.extension().and_then(|e| e.to_str()), Some("kt" | "kts")) {
                out.push(path);
            }
        }
    }
    let mut paths = Vec::new();
    walk(dir, &mut paths);
    let mut files: Vec<(String, PathBuf)> = paths
        .into_iter()
        .filter(|p| !fixture || p.with_extension("txt").is_file())
        .map(|p| (p.strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/"), p))
        .collect();
    files.sort();
    files
}

/// Report of one file as the oracle computes it.
pub fn report_file(path: &Path, fixture: bool, script: bool) -> String {
    let raw = std::fs::read_to_string(path).unwrap();
    let name = path.file_name().unwrap().to_str().unwrap();
    report(&normalize(&raw, fixture), name, script)
}

/// Report hashes for every file, on all cores (big stacks: the visitor walk recurses per tree level).
pub fn hashes(dir: &Path, fixture: bool, script: bool) -> Vec<(String, u64)> {
    let files = files(dir, fixture);
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get());
    let handles: Vec<_> = files
        .chunks(files.len().div_ceil(workers).max(1))
        .map(<[_]>::to_vec)
        .map(|chunk| {
            std::thread::Builder::new()
                .stack_size(512 << 20)
                .spawn(move || {
                    let hash = |path: &Path| fnv1a64(&report_file(path, fixture, script));
                    chunk.into_iter().map(|(rel, path)| (rel, hash(&path))).collect::<Vec<_>>()
                })
                .unwrap()
        })
        .collect();
    handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
}

/// Relative paths whose hash differs from (or is missing in) `expected` ("<rel>\t<hex>" lines).
pub fn mismatches<'a>(expected: &str, actual: &'a [(String, u64)]) -> Vec<&'a str> {
    let expected: std::collections::HashMap<&str, &str> = expected.lines().filter_map(|l| l.split_once('\t')).collect();
    actual
        .iter()
        .filter(|(rel, hash)| expected.get(rel.as_str()).copied() != Some(format!("{hash:016x}").as_str()))
        .map(|(rel, _)| rel.as_str())
        .collect()
}

pub fn fnv1a64(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
