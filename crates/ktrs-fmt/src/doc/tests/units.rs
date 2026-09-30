use crate::doc::{
    BlankLineWanted, BreakTag, Indent, Range, RangeSet, newlines, reformat_parameter_comment,
    utf16_len,
};
use crate::format::input::KotlinTok;

#[test]
fn newlines_offsets_and_lines() {
    assert_eq!(newlines::count("a\nb\r\nc\rd"), 3);
    assert_eq!(newlines::count(""), 0);
    assert_eq!(newlines::first_break("ab\ncd"), 3);
    assert_eq!(newlines::first_break("abc"), -1);
    assert_eq!(
        newlines::line_offset_iterator("a\r\nb\n").collect::<Vec<_>>(),
        [0, 3, 5]
    );
    assert_eq!(
        newlines::line_iterator("a\nb").collect::<Vec<_>>(),
        ["a\n", "b"]
    );
    assert_eq!(newlines::line_iterator("a\n").collect::<Vec<_>>(), ["a\n"]);
    assert_eq!(newlines::line_iterator("").count(), 0);
    assert_eq!(newlines::has_newline_at("x\r\n", 1), 2);
    assert_eq!(newlines::has_newline_at("x\n", 0), -1);
    assert!(newlines::is_newline("\r\n") && !newlines::is_newline("\n\n"));
    assert_eq!(newlines::guess_line_separator("a\r\nb"), "\r\n");
    assert_eq!(newlines::guess_line_separator("ab"), "\n");
    assert_eq!(newlines::get_line_ending("a\r"), Some("\r"));
}

#[test]
fn utf16_widths() {
    assert_eq!(utf16_len("abc"), 3);
    assert_eq!(utf16_len("é日"), 2);
    assert_eq!(utf16_len("😀"), 2);
}

#[test]
fn range_set_keeps_closed_ranges_apart() {
    let mut closed = RangeSet::create();
    closed.add_closed(0, 2);
    closed.add_closed(3, 5);
    assert_eq!(closed.range_containing(1), Some((0, 2)));
    assert_eq!(closed.range_containing(4), Some((3, 5)));
    closed.add_closed(2, 3);
    assert_eq!(closed.range_containing(1), Some((0, 5)));

    let mut open = RangeSet::create();
    open.add(Range::closed_open(0, 3));
    open.add(Range::closed_open(3, 6));
    open.add(Range::closed_open(-1, -1));
    assert_eq!(open.as_canonical_ranges(), [Range::closed_open(0, 6)]);
    assert_eq!(
        open.sub_range_set_closed(0, 4).as_canonical_ranges(),
        [Range::closed_open(0, 5)]
    );
}

fn comment(text: &str) -> KotlinTok<'_> {
    KotlinTok::new(0, None, text, 0, 0, false)
}

#[test]
fn parameter_comments() {
    let reformat = |text: &str| reformat_parameter_comment(&comment(text));
    assert_eq!(reformat("/* x = */").as_deref(), Some("/* x= */"));
    assert_eq!(reformat("/*name=*/").as_deref(), Some("/* name= */"));
    assert_eq!(
        reformat("/* args... =*/").as_deref(),
        Some("/* args...= */")
    );
    assert_eq!(reformat("/* $é_1 = */").as_deref(), Some("/* $é_1= */"));
    assert_eq!(reformat("/* 1x = */"), None);
    assert_eq!(reformat("/* x = y */"), None);
    assert_eq!(reformat("/* x .. = */"), None);
    assert_eq!(reformat("// x = */"), None);
}

#[test]
fn blank_line_wanted_merge() {
    let tag = BreakTag::new();
    let merged =
        BlankLineWanted::conditional(&tag).merge(BlankLineWanted::conditional(&BreakTag::new()));
    assert_eq!(merged.wanted(), None);
    tag.record_broken(true);
    assert_eq!(merged.wanted(), Some(true));
    assert_eq!(
        BlankLineWanted::conditional(&tag)
            .merge(BlankLineWanted::NO)
            .wanted(),
        Some(false)
    );
    assert_eq!(
        BlankLineWanted::YES.merge(BlankLineWanted::NO).wanted(),
        Some(true)
    );
}

#[test]
fn indent_if_follows_tag() {
    let tag = BreakTag::new();
    let indent = Indent::make_if(&tag, Indent::make_const(2, 4), Indent::ZERO);
    assert_eq!(indent.eval(), 0);
    tag.record_broken(true);
    assert_eq!(indent.eval(), 8);
}
