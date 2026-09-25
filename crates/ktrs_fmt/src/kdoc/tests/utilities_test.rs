//! Port of `UtilitiesTest.kt`.

use crate::kdoc::kstring::{KStr, ks, to_string};
use crate::kdoc::utilities::{find_same_position, get_param_name};
use crate::kdoc::{FormattingTask, KDocFormattingOptions, Paragraph};

#[test]
#[allow(non_snake_case)]
fn testFindSamePosition() {
    fn check(new_with_caret: &str, old_with_caret: &str) {
        let old_with_caret = ks(old_with_caret);
        let new_with_caret = ks(new_with_caret);
        let old_caret_index = old_with_caret.index_of_char('|' as u16, 0);
        let new_caret_index = new_with_caret.index_of_char('|' as u16, 0);
        assert!(old_caret_index != -1);
        assert!(new_caret_index != -1);
        let (o, n) = (old_caret_index as usize, new_caret_index as usize);
        let old = [&old_with_caret[..o], &old_with_caret[o + 1..]].concat();
        let new = [&new_with_caret[..n], &new_with_caret[n + 1..]].concat();
        let new_pos = find_same_position(&old, old_caret_index, &new) as usize;

        let actual = [&new[..new_pos], &ks("|"), &new[new_pos..]].concat();
        assert_eq!(to_string(&actual), to_string(&new_with_caret));
    }

    // Prefix match
    check("|/** Test\n Different Middle End */", "|/** Test2 End */");
    check("/|** Test\n Different Middle End */", "/|** Test2 End */");
    check("/*|* Test\n Different Middle End */", "/*|* Test2 End */");
    check("/**| Test\n Different Middle End */", "/**| Test2 End */");
    check("/** |Test\n Different Middle End */", "/** |Test2 End */");
    check("/** T|est\n Different Middle End */", "/** T|est2 End */");
    check("/** Te|st\n Different Middle End */", "/** Te|st2 End */");
    check("/** Tes|t\n Different Middle End */", "/** Tes|t2 End */");
    check("/** Test|\n Different Middle End */", "/** Test|2 End */");
    // End match
    check("/** Test\n Different Middle| End */", "/** Test2| End */");
    check("/** Test\n Different Middle E|nd */", "/** Test2 E|nd */");
    check("/** Test\n Different Middle En|d */", "/** Test2 En|d */");
    check("/** Test\n Different Middle End| */", "/** Test2 End| */");
    check("/** Test\n Different Middle End |*/", "/** Test2 End |*/");
    check("/** Test\n Different Middle End *|/", "/** Test2 End *|/");
    check("/** Test\n Different Middle End */|", "/** Test2 End */|");

    check("|/**\nTest End\n*/", "|/** Test End */");
    check("/|**\nTest End\n*/", "/|** Test End */");
    check("/*|*\nTest End\n*/", "/*|* Test End */");
    check("/**|\nTest End\n*/", "/**| Test End */");
    check("/**\n|Test End\n*/", "/** |Test End */");
    check("/**\nT|est End\n*/", "/** T|est End */");
    check("/**\nTe|st End\n*/", "/** Te|st End */");
    check("/**\nTes|t End\n*/", "/** Tes|t End */");
    check("/**\nTest| End\n*/", "/** Test| End */");
    check("/**\nTest |End\n*/", "/** Test |End */");
    check("/**\nTest E|nd\n*/", "/** Test E|nd */");
    check("/**\nTest En|d\n*/", "/** Test En|d */");
    check("/**\nTest End|\n*/", "/** Test End| */");
    check("/**\nTest End\n|*/", "/** Test End |*/");
    check("/**\nTest End\n*|/", "/** Test End *|/");
    check("/**\nTest End\n*/|", "/** Test End */|");

    check("|/** Test End */", "|/** Test2 End */");
    check("/|** Test End */", "/|** Test2 End */");
    check("/*|* Test End */", "/*|* Test2 End */");
    check("/**| Test End */", "/**| Test2 End */");
    check("/** |Test End */", "/** |Test2 End */");
    check("/** T|est End */", "/** T|est2 End */");
    check("/** Te|st End */", "/** Te|st2 End */");
    check("/** Tes|t End */", "/** Tes|t2 End */");
    check("/** Test| End */", "/** Test|2 End */");
    check("/** Test |End */", "/** Test2 |End */");
    check("/** Test E|nd */", "/** Test2 E|nd */");
    check("/** Test En|d */", "/** Test2 En|d */");
    check("/** Test End| */", "/** Test2 End| */");
    check("/** Test End |*/", "/** Test2 End |*/");
    check("/** Test End *|/", "/** Test2 End *|/");
    check("/** Test End */|", "/** Test2 End */|");
}

#[test]
#[allow(non_snake_case)]
fn testGetParamName() {
    let name = |s: &str| get_param_name(&ks(s)).map(to_string);
    assert_eq!(name("@param foo").as_deref(), Some("foo"));
    assert_eq!(name("@param foo bar").as_deref(), Some("foo"));
    assert_eq!(name("@param foo;").as_deref(), Some("foo"));
    assert_eq!(name("  \t@param\t   foo  bar.").as_deref(), Some("foo"));
    assert_eq!(name("@param[foo]").as_deref(), Some("foo"));
    assert_eq!(name("@param  [foo]").as_deref(), Some("foo"));
    assert_eq!(name("@param "), None);
    assert_eq!(name("@property foo").as_deref(), Some("foo"));
}

#[test]
#[allow(non_snake_case)]
fn testComputeWords() {
    fn describe(list: &[String]) -> String {
        format!("listOf({})", list.iter().map(|it| format!("\"{it}\"")).collect::<Vec<_>>().join(", "))
    }
    fn check_with(text: &str, expected: &[&str], customize_paragraph: impl Fn(&mut Paragraph)) {
        let task = FormattingTask::new(KDocFormattingOptions::new(12, 12), &format!("/** {text} */"), "");
        let mut paragraph = Paragraph::new(&task);
        paragraph.content.extend(ks(text));
        customize_paragraph(&mut paragraph);
        let words: Vec<String> = paragraph.compute_words().iter().map(|w| to_string(w)).collect();
        let expected: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(describe(&words), describe(&expected));
    }
    let check = |text: &str, expected: &[&str]| check_with(text, expected, |_| {});

    check("Foo", &["Foo"]);
    check("Foo Bar Baz", &["Foo", "Bar", "Baz"]);
    check_with("Foo Bar Baz", &["Foo Bar", "Baz"], |it| it.quoted = 1);
    check_with("Foo Bar Baz", &["Foo Bar", "Baz"], |it| it.set_hanging(true));
    check("1. Foo", &["1.", "Foo"]);
    // "1." can't start a line; there it would become a numbered element.
    check("Foo 1.", &["Foo 1."]);
    check("Foo bar [Link Text] foo bar.", &["Foo", "bar", "[Link Text]", "foo", "bar."]);
    check("Interval [0, 1) foo bar.", &["Interval [0, 1)", "foo", "bar."]);

    // ">" cannot start a word; it would become quoted text
    check("if >= 3", &["if >=", "3"]);
    check("if >= 3.", &["if >= 3."]);

    check(
        "SDK version - [`Partial(Mode.UseIfAvailable)`](Partial) on API 24+",
        &["SDK", "version - [`Partial(Mode.UseIfAvailable)`](Partial)", "on", "API", "24+"],
    );

    check(
        "Z orders can range from Integer.MIN_VALUE to Integer.MAX_VALUE. Default z order  index is 0. \
         [SurfaceControlWrapper] instances are positioned back-to-front.",
        &[
            "Z", "orders", "can", "range", "from", "Integer.MIN_VALUE", "to", "Integer.MAX_VALUE.",
            "Default", "z", "order", "index", "is 0. [SurfaceControlWrapper]", "instances", "are",
            "positioned", "back-to-front.",
        ],
    );
    check(
        "Equates to `cmd package compile -f -m speed <package>` on API 24+.",
        &[
            "Equates", "to", "`cmd", "package", "compile", "-f", "-m", "speed", "<package>`", "on", "API",
            "24+.",
        ],
    );
}
