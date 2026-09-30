//! Port of ktfmt's `KotlinInputTest.kt` and `WhitespaceTombstonesTest.kt`.

use ktrs_parser::FileKind;

use crate::doc::Input;
use crate::format::input::KotlinInput;
use crate::format::input::whitespace_tombstones::{
    SPACE_TOMBSTONE, replace_trailing_whitespace_with_tombstone,
};

#[test]
fn comments_are_toks_not_tokens() {
    let code = "/** foo */ class F {}";
    let input = KotlinInput::new(
        code,
        &ktrs_psi::PsiElement::root(ktrs_parser::parse_file(code, FileKind::Script).tree),
    )
    .unwrap();
    let tokens: Vec<&str> = input
        .get_tokens()
        .iter()
        .map(|t| t.get_tok().get_text())
        .collect();
    assert_eq!(tokens, ["class", "F", "{", "}", ""]);
    let before: Vec<&str> = input.get_tokens()[0]
        .get_toks_before()
        .iter()
        .map(|t| t.get_text())
        .collect();
    assert_eq!(before, ["/** foo */", " "]);
}

#[test]
fn test_replace_trailing_whitespace_with_tombstone() {
    let t = SPACE_TOMBSTONE;
    let r = replace_trailing_whitespace_with_tombstone;
    assert_eq!(r(""), "");
    assert_eq!(r("  sdfl"), "  sdfl");
    assert_eq!(r("  sdfl "), format!("  sdfl{t}"));
    assert_eq!(r("  sdfl  "), format!("  sdfl {t}"));
    assert_eq!(r("  sdfl  \n skdjfh"), format!("  sdfl {t}\n skdjfh"));
    assert_eq!(r("  sdfl  \n skdjfh "), format!("  sdfl {t}\n skdjfh{t}"));
    assert_eq!(
        r("  sdfl  \n\n skdjfh "),
        format!("  sdfl {t}\n\n skdjfh{t}")
    );
}
