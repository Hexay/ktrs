//! Interpreter for the op scripts in `cases.txt`; mirrors `tools/ktfmt-oracle/engine/DocScript.java`,
//! which runs the same scripts through gjf's real `OpsBuilder`/`Doc`/`JavaOutput`.
//!
//! Commands: `t`/`tN` emit the next 1/N tokens, `tc I`/`tt I` a token with comment/trailing
//! indent, `r TEXT` a real token, `g TEXT` a guessed token, `o I` / `oif TAG THEN ELSE` open,
//! `c` close, `s` space, `b MODE FLAT I` / `bt MODE FLAT I TAG` breaks (MODE `U|I|F`, FLAT `_`
//! empty or `sp` space), `bl yes|no|preserve|TAG`, `fence`, `mark`, `sync POS`.

use std::collections::HashMap;

use ktrs_parser::FileKind;

use crate::doc::{
    BlankLineWanted, BreakTag, CommentsHelper, DocBuilder, FillMode, Indent, JavaOutput, Op,
    OpsBuilder, Output, Range, RealOrImaginary, State, Tok,
};
use crate::format::input::KotlinInput;
use crate::format::input::whitespace_tombstones::replace_tombstone_with_trailing_whitespace;

pub(super) struct IdentityCommentsHelper;

impl CommentsHelper for IdentityCommentsHelper {
    fn rewrite(&self, tok: &dyn Tok, _max_width: i32, _column0: i32) -> String {
        tok.get_original_text().to_string()
    }
}

fn mode(m: &str) -> FillMode {
    match m {
        "U" => FillMode::Unified,
        "I" => FillMode::Independent,
        _ => FillMode::Forced,
    }
}

fn flat(f: &str) -> &str {
    match f {
        "_" => "",
        "sp" => " ",
        f => f,
    }
}

fn run_script(b: &mut OpsBuilder<'_>, script: &str) {
    let mut tags: HashMap<String, BreakTag> = HashMap::new();
    let mut tag = |name: &str| tags.entry(name.to_string()).or_default().clone();
    let mut it = script.split_whitespace();
    let int = |it: &mut std::str::SplitWhitespace<'_>| it.next().unwrap().parse::<i32>().unwrap();
    while let Some(cmd) = it.next() {
        if let Some(n) = cmd
            .strip_prefix('t')
            .filter(|n| n.chars().all(|c| c.is_ascii_digit()))
        {
            for _ in 0..n.parse().unwrap_or(1) {
                b.token(
                    b.peek_token().unwrap(),
                    RealOrImaginary::Real,
                    Indent::ZERO,
                    None,
                );
            }
            continue;
        }
        match cmd {
            "o" => b.open(Indent::make_const(int(&mut it), 1)),
            "oif" => {
                let t = tag(it.next().unwrap());
                let then_indent = Indent::make_const(int(&mut it), 1);
                let else_indent = Indent::make_const(int(&mut it), 1);
                b.open(Indent::make_if(&t, then_indent, else_indent));
            }
            "c" => b.close(),
            "tc" => b.token(
                b.peek_token().unwrap(),
                RealOrImaginary::Real,
                Indent::make_const(int(&mut it), 1),
                None,
            ),
            "tt" => {
                let trailing = Some(Indent::make_const(int(&mut it), 1));
                b.token(
                    b.peek_token().unwrap(),
                    RealOrImaginary::Real,
                    Indent::ZERO,
                    trailing,
                );
            }
            "s" => b.space(),
            "b" => {
                let m = mode(it.next().unwrap());
                let f = flat(it.next().unwrap());
                b.break_op(m, f, Indent::make_const(int(&mut it), 1));
            }
            "bt" => {
                let m = mode(it.next().unwrap());
                let f = flat(it.next().unwrap());
                let indent = Indent::make_const(int(&mut it), 1);
                b.break_op_tagged(m, f, indent, Some(tag(it.next().unwrap())));
            }
            "bl" => b.blank_line_wanted(match it.next().unwrap() {
                "yes" => BlankLineWanted::YES,
                "no" => BlankLineWanted::NO,
                "preserve" => BlankLineWanted::PRESERVE,
                t => BlankLineWanted::conditional(&tag(t)),
            }),
            "g" => b.guess_token(it.next().unwrap()),
            "r" => b.token(
                it.next().unwrap(),
                RealOrImaginary::Real,
                Indent::ZERO,
                None,
            ),
            "sync" => b.sync(int(&mut it)),
            "fence" => b.add(Op::FenceComments),
            "mark" => b.mark_for_partial_format(),
            _ => panic!("bad command {cmd}"),
        }
    }
}

/// Formats `code` like ktfmt's `Formatter.prettyPrint`, with the visitor replaced by `script`.
pub(super) fn run(code: &str, width: i32, script: &str) -> String {
    let parse = ktrs_parser::parse_file(code, FileKind::Script);
    assert!(
        !parse.has_errors(),
        "parse errors: {:?}",
        parse.error_messages
    );
    let input = KotlinInput::new(code, &parse.syntax()).unwrap();
    let mut output = JavaOutput::new("\n", &input, Box::new(IdentityCommentsHelper));
    let mut builder = OpsBuilder::new(&input, &mut output);
    builder.mark_for_partial_format();
    run_script(&mut builder, script);
    builder.sync(code.len() as i32);
    builder.drain();
    let ops = match builder.build() {
        Ok(ops) => ops,
        Err(e) => return format!("ERROR: {e}"),
    };
    let mut doc = DocBuilder::new().with_ops(ops).build();
    doc.compute_breaks(output.get_comments_helper(), width, State::new(0, 0));
    doc.write(&mut output);
    output.flush();
    let ranges = input
        .character_ranges_to_token_ranges(&[Range::closed_open(0, code.len() as i32)])
        .unwrap();
    let replacements = output.get_format_replacements(&ranges);
    replace_tombstone_with_trailing_whitespace(&JavaOutput::apply_replacements(code, &replacements))
}
