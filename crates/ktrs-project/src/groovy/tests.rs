use super::parse;
use crate::ir::{Expr, Stmt};

fn names(e: &Expr) -> Vec<&str> {
    match e {
        Expr::Path(segs) => segs.iter().map(|s| s.name.as_str()).collect(),
        _ => Vec::new(),
    }
}

#[test]
fn parses_commands_closures_and_assignments() {
    let stmts = parse(
        "plugins {\n  id 'org.x' version '1.0' apply false\n}\napply plugin: \"a.b\"\ndef v = \"1.8.0\"\n\
         ktlint {\n  version = v // c\n  additionalEditorconfig = ['k': 'v']\n}\nfoo.bar(1, x: 2) { it -> baz() }\n",
    );
    let Stmt::Expr(Expr::Path(plugins)) = &stmts[0] else { panic!("{stmts:?}") };
    let Stmt::Expr(id) = &plugins[0].block.as_ref().unwrap()[0] else { panic!() };
    assert_eq!(names(id), ["id", "version", "apply"]);
    let Stmt::Expr(Expr::Path(apply)) = &stmts[1] else { panic!() };
    assert_eq!(apply[0].named_arg("plugin"), Some(&Expr::Str("a.b".into())));
    assert_eq!(stmts[2], Stmt::Val("v".into(), Expr::Str("1.8.0".into())));
    let Stmt::Expr(Expr::Path(ktlint)) = &stmts[3] else { panic!() };
    let body = ktlint[0].block.as_ref().unwrap();
    assert!(matches!(&body[0], Stmt::Assign(l, Expr::Path(_)) if names(l) == ["version"]));
    assert!(matches!(&body[1], Stmt::Assign(_, Expr::List(items)) if items.len() == 1));
    let Stmt::Expr(foo) = &stmts[4] else { panic!() };
    assert_eq!(names(foo), ["foo", "bar"]);
}

#[test]
fn interpolates_gstrings_and_survives_garbage() {
    let stmts = parse("x = \"a:${v}:$w.y\"\n) } ] 'unterminated\nfoo { bar 1 + 2 }\n");
    let Stmt::Assign(_, Expr::Template(parts)) = &stmts[0] else { panic!("{stmts:?}") };
    assert_eq!(parts.len(), 5);
    assert!(stmts.iter().any(|s| matches!(s, Stmt::Expr(e) if names(e) == ["foo"])));
}
