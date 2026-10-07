//! Kotlin build scripts and convention sources -> IR, through ktrs-parser's tree.

use ktrs_parser::{FileKind, parse_file};
use ktrs_psi::{KtFile, PsiElement};
use ktrs_syntax::SyntaxKind::{self, *};

use crate::ir::{Arg, Expr, Part, Seg, Stmt};

pub(crate) fn parse(text: &str, script: bool) -> Vec<Stmt> {
    let parse = parse_file(text, if script { FileKind::Script } else { FileKind::Source });
    let file = KtFile::new(&parse);
    let mut out = Vec::new();
    stmts_in(&file, &mut out);
    out
}

fn child(e: &PsiElement, kind: SyntaxKind) -> Option<PsiElement> {
    e.all_children().find(|c| c.element_type() == kind)
}

fn composites(e: &PsiElement) -> impl Iterator<Item = PsiElement> {
    e.all_children().filter(|c| !c.is_leaf() && !is_comment(c.element_type()))
}

fn is_comment(kind: SyntaxKind) -> bool {
    matches!(kind, EOL_COMMENT | BLOCK_COMMENT | DOC_COMMENT | SHEBANG_COMMENT)
}

fn identifier(e: &PsiElement) -> Option<String> {
    child(e, IDENTIFIER).map(|i| unquote(&i.text()))
}

fn unquote(name: &str) -> String {
    name.trim_matches('`').to_string()
}

fn stmts_in(e: &PsiElement, out: &mut Vec<Stmt>) {
    for c in composites(e) {
        stmt(&c, out);
    }
}

fn stmt(e: &PsiElement, out: &mut Vec<Stmt>) {
    match e.element_type() {
        IMPORT_LIST | PACKAGE_DIRECTIVE | MODIFIER_LIST | FILE_ANNOTATION_LIST => {}
        PROPERTY => {
            let Some(name) = identifier(e) else { return };
            let value = if let Some(delegate) = child(e, PROPERTY_DELEGATE) {
                let Some(delegated) = composites(&delegate).next() else { return };
                if matches!(delegated.text().as_str(), "project" | "rootProject" | "extra") {
                    // `val x: String by project`: a Gradle property of the same name.
                    let lookup = Seg { args: Some(vec![arg(None, Expr::Str(name.clone()))]), ..Seg::named("property") };
                    return out.push(Stmt::Val(name, Expr::Path(vec![lookup])));
                }
                expr(&delegated)
            } else if let Some(init) = after_eq(e) {
                expr(&init)
            } else {
                return;
            };
            // `val check by tasks.registering(JavaExec::class) { .. }` still runs its block.
            if let Expr::Path(segs) = &value
                && segs.iter().any(|s| s.block.is_some())
            {
                out.push(Stmt::Expr(value.clone()));
            }
            out.push(Stmt::Val(name, value));
        }
        FUN => {
            let Some(name) = identifier(e) else { return };
            let mut body = Vec::new();
            if let Some(block) = child(e, BLOCK) {
                stmts_in(&block, &mut body);
            } else if let Some(init) = after_eq(e) {
                stmt(&init, &mut body);
            }
            out.push(Stmt::Fun(name, body));
        }
        CLASS | OBJECT_DECLARATION => {
            let mut body = Vec::new();
            if let Some(class_body) = child(e, CLASS_BODY) {
                stmts_in(&class_body, &mut body);
            }
            out.push(Stmt::Class(identifier(e).unwrap_or_default(), body));
        }
        BINARY_EXPRESSION if operation(e).as_deref() == Some("=") => {
            let mut sides = composites(e).filter(|c| c.element_type() != OPERATION_REFERENCE);
            if let (Some(l), Some(r)) = (sides.next(), sides.next()) {
                out.push(Stmt::Assign(expr(&l), expr(&r)));
            }
        }
        CALL_EXPRESSION | DOT_QUALIFIED_EXPRESSION | SAFE_ACCESS_EXPRESSION | REFERENCE_EXPRESSION
        | BINARY_EXPRESSION => out.push(Stmt::Expr(expr(e))),
        // `if`/`when`/`try` bodies, script blocks, annotated statements: flattened.
        _ => stmts_in(e, out),
    }
}

/// The expression after a declaration's `=`.
fn after_eq(e: &PsiElement) -> Option<PsiElement> {
    e.all_children()
        .skip_while(|c| c.element_type() != EQ)
        .find(|c| !c.is_leaf() && !is_comment(c.element_type()))
}

fn operation(e: &PsiElement) -> Option<String> {
    child(e, OPERATION_REFERENCE).map(|o| o.text())
}

fn arg(name: Option<String>, value: Expr) -> Arg {
    Arg { name, value }
}

fn expr(e: &PsiElement) -> Expr {
    match e.element_type() {
        STRING_TEMPLATE => template(e),
        INTEGER_CONSTANT => {
            let digits: String = e.text().chars().filter(|c| c.is_ascii_digit()).collect();
            digits.parse().map(Expr::Int).unwrap_or(Expr::Other)
        }
        BOOLEAN_CONSTANT => Expr::Bool(e.text() == "true"),
        PARENTHESIZED | ANNOTATED_EXPRESSION | LABELED_EXPRESSION => {
            composites(e).filter(|c| c.element_type() != ANNOTATION_ENTRY).last().map_or(Expr::Other, |c| expr(&c))
        }
        REFERENCE_EXPRESSION | THIS_EXPRESSION | CLASS_LITERAL_EXPRESSION => {
            Expr::Path(vec![Seg::named(unquote(&e.text()))])
        }
        CALL_EXPRESSION => Expr::Path(vec![call(e)]),
        DOT_QUALIFIED_EXPRESSION | SAFE_ACCESS_EXPRESSION => {
            let mut parts = composites(e);
            let mut segs = path_of(parts.next());
            segs.extend(path_of(parts.next()));
            Expr::Path(segs)
        }
        BINARY_EXPRESSION => {
            let mut sides = composites(e).filter(|c| c.element_type() != OPERATION_REFERENCE);
            let (Some(l), Some(r)) = (sides.next(), sides.next()) else { return Expr::Other };
            match operation(e).as_deref() {
                Some("to") => Expr::Pair(Box::new(expr(&l)), Box::new(expr(&r))),
                // Infix calls: `id("x") version "1"`, `alias(..) apply false`.
                Some(op) if op.chars().all(|c| c.is_alphanumeric() || c == '_') => {
                    let mut segs = path_of(Some(l));
                    segs.push(Seg { args: Some(vec![arg(None, expr(&r))]), ..Seg::named(op) });
                    Expr::Path(segs)
                }
                _ => Expr::Other,
            }
        }
        _ => Expr::Other,
    }
}

fn path_of(e: Option<PsiElement>) -> Vec<Seg> {
    match e.map(|e| expr(&e)) {
        Some(Expr::Path(segs)) => segs,
        _ => vec![Seg::named("?")],
    }
}

fn call(e: &PsiElement) -> Seg {
    let mut seg = Seg::default();
    let mut args = Vec::new();
    for c in composites(e) {
        match c.element_type() {
            TYPE_ARGUMENT_LIST => {
                seg.type_args = composites(&c).map(|t| t.text()).collect();
            }
            VALUE_ARGUMENT_LIST => {
                for a in composites(&c).filter(|a| a.element_type() == VALUE_ARGUMENT) {
                    let name = child(&a, VALUE_ARGUMENT_NAME).map(|n| unquote(n.text().trim()));
                    let value = composites(&a).find(|v| v.element_type() != VALUE_ARGUMENT_NAME);
                    args.push(arg(name, value.map_or(Expr::Other, |v| expr(&v))));
                }
            }
            LAMBDA_ARGUMENT => seg.block = lambda_body(&c),
            _ => {
                seg.name = match expr(&c) {
                    Expr::Path(segs) if segs.len() == 1 => segs.into_iter().next().map(|s| s.name).unwrap_or_default(),
                    Expr::Str(s) => s,
                    _ => "?".into(),
                }
            }
        }
    }
    seg.args = Some(args);
    seg
}

fn lambda_body(e: &PsiElement) -> Option<Vec<Stmt>> {
    if e.element_type() == LAMBDA_EXPRESSION {
        let block = child(&child(e, FUNCTION_LITERAL)?, BLOCK)?;
        let mut out = Vec::new();
        stmts_in(&block, &mut out);
        return Some(out);
    }
    composites(e).find_map(|c| lambda_body(&c))
}

fn template(e: &PsiElement) -> Expr {
    let mut parts = Vec::new();
    for entry in composites(e) {
        match entry.element_type() {
            LITERAL_STRING_TEMPLATE_ENTRY => parts.push(Part::Lit(entry.text())),
            ESCAPE_STRING_TEMPLATE_ENTRY => parts.push(Part::Lit(unescape(&entry.text()))),
            _ => parts.push(Part::Expr(composites(&entry).next().map_or(Expr::Other, |v| expr(&v)))),
        }
    }
    if parts.iter().all(|p| matches!(p, Part::Lit(_))) {
        Expr::Str(parts.into_iter().map(|p| if let Part::Lit(s) = p { s } else { String::new() }).collect())
    } else {
        Expr::Template(parts)
    }
}

fn unescape(s: &str) -> String {
    match s {
        "\\n" => "\n".into(),
        "\\t" => "\t".into(),
        "\\r" => "\r".into(),
        _ => s.trim_start_matches('\\').into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_calls_infix_and_lambdas() {
        let stmts = parse(
            "plugins { id(\"a.b\") version \"1\" apply false }\nval v = \"x$y\"\nktfmt { maxWidth.set(120); blockIndent = 4 }",
            true,
        );
        let Stmt::Expr(Expr::Path(plugins)) = &stmts[0] else { panic!("{stmts:?}") };
        let Some(Stmt::Expr(Expr::Path(id))) = plugins[0].block.as_ref().and_then(|b| b.first()) else { panic!() };
        let names: Vec<&str> = id.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["id", "version", "apply"]);
        assert_eq!(id[2].arg(0), Some(&Expr::Bool(false)));
        assert!(matches!(&stmts[1], Stmt::Val(n, Expr::Template(_)) if n == "v"));
        let Stmt::Expr(Expr::Path(ktfmt)) = &stmts[2] else { panic!() };
        let block = ktfmt[0].block.as_ref().unwrap();
        assert!(matches!(&block[0], Stmt::Expr(Expr::Path(p)) if p[1].name == "set"));
        assert!(matches!(&block[1], Stmt::Assign(_, Expr::Int(4))));
    }
}
