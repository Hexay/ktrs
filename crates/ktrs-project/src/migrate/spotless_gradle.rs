//! Spotless's Gradle `ktfmt()` / `ktlint()` steps -> `addStep(KtrsStep..)` / `addStep(KtrsKtlintStep..)`, plus
//! the `io.github.hexay:ktrs` classpath entry (README "Integrations > Gradle", Spotless).

use super::Notes;
use super::VERSIONS_2_0_AND_1_8;
use super::coords::{JAR_ARTIFACT, JAR_GROUP};
use super::edits::{Edits, append_to_block, indent_at, indent_unit, insert_at_top, starts_line};
use super::scan::{Kind, Script};

const KTRS_STEP: &str = "addStep(io.github.hexay.ktrs.spotless.KtrsStep.create(io.github.hexay.ktrs.KtrsOptions";
const KTLINT_STEP: &str =
    "addStep(io.github.hexay.ktrs.spotless.KtrsKtlintStep.create(io.github.hexay.ktrs.KtlintOptions";
const DEFAULT_EDITORCONFIG: &str = ".withEditorConfigPath(rootProject.file(\".editorconfig\"))";

/// One `.name(args)` / `.name { }` link of a step's call chain.
struct Link {
    name_tok: usize,
    /// Tokens inside the parentheses or braces.
    args: std::ops::Range<usize>,
}

/// Resolves a version expression's source text (`ktlintVersion`, `libs.versions.ktlint.get()`).
pub(crate) type Resolve<'r> = &'r dyn Fn(&str) -> Option<String>;

/// Rewrites the file's Spotless steps; `true` when any was rewritten (the jar must then be on the classpath).
/// `in_convention`: the file can't have a `buildscript {}`; steps there only get a note.
pub(crate) fn rewrite(s: &Script, resolve: Resolve, in_convention: bool, edits: &mut Edits, notes: &mut Notes) -> bool {
    let steps: Vec<usize> = (0..s.toks.len()).filter(|&i| is_step(s, i)).collect();
    if steps.is_empty() {
        return false;
    }
    if in_convention {
        notes.gradle(format!(
            "Spotless ktfmt()/ktlint() in a convention plugin: add {JAR_GROUP}:{JAR_ARTIFACT} to its build's dependencies and switch to KtrsStep/KtrsKtlintStep by hand"
        ));
        return false;
    }
    if let Some(v) = spotless_version(s).filter(|v| v.split('.').next().and_then(|m| m.parse::<u32>().ok()) < Some(7)) {
        notes.gradle(format!("Spotless {v}: KtrsStep and KtrsKtlintStep need Spotless 7+; upgrade it, then rerun"));
        return false;
    }
    let mut any = false;
    for i in steps {
        let (end, links) = chain(s, i);
        let result = if s.src(i) == "ktfmt" { ktfmt(s, &links, resolve) } else { ktlint(s, &links, resolve) };
        match result {
            Ok(step_edits) => {
                edits.extend(step_edits.finish(s, i, end));
                any = true;
            }
            Err(why) => notes.gradle(format!("Spotless {}(): {why}; switch it by hand", s.src(i))),
        }
    }
    any
}

fn is_step(s: &Script, i: usize) -> bool {
    if !(s.is_ident(i, "ktfmt") || s.is_ident(i, "ktlint"))
        || !s.is_punct(i + 1, '(')
        || s.is_punct(i.wrapping_sub(1), '.')
    {
        return false;
    }
    let blocks = s.enclosing(i);
    let Some(&inner) = blocks.first() else { return false };
    matches!(s.block_name(inner), Some("kotlin" | "kotlinGradle"))
        && blocks
            .iter()
            .any(|&o| s.block_name(o) == Some("spotless") || s.block_type_arg(o) == Some("SpotlessExtension"))
}

/// `com.diffplug.spotless` declared with a literal version in this file.
fn spotless_version<'a>(s: &Script<'a>) -> Option<&'a str> {
    let i = (0..s.toks.len()).find(|&i| s.plain_str(i) == Some("com.diffplug.spotless"))?;
    let j = if s.is_punct(i + 1, ')') { i + 2 } else { i + 1 };
    s.is_ident(j, "version").then(|| s.plain_str(if s.is_punct(j + 1, '(') { j + 2 } else { j + 1 }))?
}

/// The links after `name(..)` at `i`, and the index after the chain.
fn chain(s: &Script, i: usize) -> (usize, Vec<Link>) {
    let mut links = vec![Link { name_tok: i, args: i + 2..s.skip_group(i + 1) - 1 }];
    let mut j = s.skip_group(i + 1);
    while s.is_punct(j, '.') && s.toks.get(j + 1).is_some_and(|t| t.kind == Kind::Ident) {
        let open = j + 2;
        if !(s.is_punct(open, '(') || s.is_punct(open, '{')) {
            break;
        }
        let after = s.skip_group(open);
        links.push(Link { name_tok: j + 1, args: open + 1..after - 1 });
        j = after;
    }
    (j, links)
}

fn args_src<'a>(s: &Script<'a>, r: &std::ops::Range<usize>) -> &'a str {
    if r.is_empty() { "" } else { &s.text[s.toks[r.start].span.start..s.toks[r.end - 1].span.end] }
}

/// The version argument: `None` when absent, else (source, resolved value).
fn version_arg(s: &Script, link: &Link, resolve: Resolve) -> Result<Option<(String, String)>, String> {
    let src = args_src(s, &link.args);
    if src.is_empty() {
        return Ok(None);
    }
    let value = if link.args.len() == 1 { s.plain_str(link.args.start).map(str::to_string) } else { None };
    match value.or_else(|| resolve(src)) {
        Some(v) => Ok(Some((src.to_string(), v))),
        None => Err(format!("can't resolve the version `{src}`")),
    }
}

enum StepEdits {
    /// Replace the whole chain.
    Whole(String),
    /// Replace the head call, rename links in place, close the two calls at the end.
    InPlace { head: String, renames: Vec<(usize, String)>, arg_wraps: Vec<(std::ops::Range<usize>, String)> },
}

impl StepEdits {
    fn finish(self, s: &Script, i: usize, end: usize) -> Edits {
        let mut e = Edits::default();
        let start = s.toks[i].span.start;
        let chain_end = s.toks[end - 1].span.end;
        match self {
            StepEdits::Whole(text) => e.replace(start..chain_end, text),
            StepEdits::InPlace { head, renames, arg_wraps } => {
                e.replace(start..s.toks[s.skip_group(i + 1) - 1].span.end, head);
                for (tok, name) in renames {
                    e.replace(s.toks[tok].span.clone(), name);
                }
                for (range, text) in arg_wraps {
                    e.replace(range, text);
                }
                e.insert(chain_end, "))");
            }
        }
        e
    }
}

fn ktfmt(s: &Script, links: &[Link], resolve: Resolve) -> Result<StepEdits, String> {
    if let Some((_, v)) = version_arg(s, &links[0], resolve)?
        && !(v == "0.64" || v.starts_with("0.64."))
    {
        return Err(format!("ktfmt {v}: ktrs formats as ktfmt 0.64"));
    }
    let mut style = "meta";
    let mut options = String::new();
    for link in &links[1..] {
        match s.src(link.name_tok) {
            "kotlinlangStyle" | "dropboxStyle" => style = "kotlinlang",
            "googleStyle" => style = "google",
            "metaStyle" => style = "meta",
            "configure" => options.push_str(&configure(s, &link.args)?),
            other => return Err(format!("`.{other}(..)` has no KtrsOptions equivalent here")),
        }
    }
    Ok(StepEdits::Whole(format!("{KTRS_STEP}.{style}(){options}))")))
}

/// `configure { it.setMaxWidth(100); .. }` -> `.withMaxWidth(100)..`.
fn configure(s: &Script, body: &std::ops::Range<usize>) -> Result<String, String> {
    let mut j = body.start;
    let mut param = "it";
    if s.is_punct(j, '{') {
        return configure(s, &(j + 1..s.close_of(j)));
    }
    if s.toks.get(j).is_some_and(|t| t.kind == Kind::Ident) && s.is_punct(j + 1, '-') && s.is_punct(j + 2, '>') {
        param = s.src(j);
        j += 3;
    }
    let mut out = String::new();
    while j < body.end {
        if s.is_punct(j, ';') {
            j += 1;
            continue;
        }
        if !(s.is_ident(j, param) && s.is_punct(j + 1, '.')) {
            return Err(format!("`{}` in configure {{}} isn't a ktfmt option setter", s.src(j)));
        }
        let name = s.src(j + 2);
        let (value, next) = if s.is_punct(j + 3, '(') {
            let after = s.skip_group(j + 3);
            (args_src(s, &(j + 4..after - 1)), after)
        } else if s.is_punct(j + 3, '=') {
            let end = (j + 4..body.end)
                .find(|&k| k > j + 4 && s.is_ident(k, param) && s.is_punct(k + 1, '.'))
                .unwrap_or(body.end);
            (args_src(s, &(j + 4..end)), end)
        } else {
            return Err(format!("`{name}` in configure {{}} isn't a ktfmt option setter"));
        };
        out.push_str(&option(name, value.trim_end_matches(';').trim())?);
        j = next;
    }
    Ok(out)
}

fn option(name: &str, value: &str) -> Result<String, String> {
    let bare = name.strip_prefix("set").unwrap_or(name);
    let bare = format!("{}{}", bare[..1].to_ascii_uppercase(), &bare[1..]);
    let trailing = |v: &str| format!(".withTrailingCommas(io.github.hexay.ktrs.KtrsOptions.TrailingCommas.{v})");
    match bare.as_str() {
        "MaxWidth" | "BlockIndent" | "ContinuationIndent" | "RemoveUnusedImports" => {
            Ok(format!(".with{bare}({value})"))
        }
        "ManageTrailingCommas" if value == "true" => Ok(trailing("COMPLETE")),
        "ManageTrailingCommas" if value == "false" => Ok(trailing("NONE")),
        "TrailingCommaManagementStrategy" => match value.rsplit('.').next().unwrap_or(value) {
            v @ ("NONE" | "ONLY_ADD" | "COMPLETE") => Ok(trailing(v)),
            _ => Err(format!("can't map `{value}` to KtrsOptions.TrailingCommas")),
        },
        _ => Err(format!("`{name}({value})` has no KtrsOptions equivalent")),
    }
}

fn ktlint(s: &Script, links: &[Link], resolve: Resolve) -> Result<StepEdits, String> {
    let base = match version_arg(s, &links[0], resolve)? {
        None => "defaults()".to_string(),
        Some((_, v)) if !VERSIONS_2_0_AND_1_8.contains(&v.as_str()) => {
            return Err(format!("ktlint {v}: KtrsKtlintStep runs only 1.8.0 or 2.0.0-ALPHA-4"));
        }
        Some((src, v)) if v == "1.8.0" && src.len() == v.len() + 2 => "defaults()".to_string(),
        Some((src, _)) => format!("of({src})"),
    };
    let mut renames = Vec::new();
    let mut arg_wraps = Vec::new();
    let mut editorconfig_path = false;
    for link in &links[1..] {
        match s.src(link.name_tok) {
            "editorConfigOverride" => renames.push((link.name_tok, "withEditorConfigOverride".to_string())),
            "setEditorConfigPath" => {
                editorconfig_path = true;
                renames.push((link.name_tok, "withEditorConfigPath".to_string()));
                if s.toks.get(link.args.start).is_some_and(|t| matches!(t.kind, Kind::Str { .. })) {
                    let src = args_src(s, &link.args);
                    let span = s.toks[link.args.start].span.start..s.toks[link.args.end - 1].span.end;
                    arg_wraps.push((span, format!("file({src})")));
                }
            }
            "customRuleSets" => {
                return Err(
                    "customRuleSets takes coordinates, KtrsKtlintStep takes jar files (only compose-rules runs)".into(),
                );
            }
            other => return Err(format!("`.{other}(..)` has no KtlintOptions equivalent")),
        }
    }
    let mut path = if editorconfig_path { "" } else { DEFAULT_EDITORCONFIG };
    // A chain that continues on the next line gets the path as its own first link, as in the README.
    let next_line_dot = links.get(1).map(|l| s.toks[l.name_tok - 1].span.start).filter(|&d| starts_line(s.text, d));
    if let Some(dot) = next_line_dot.filter(|_| !path.is_empty()) {
        arg_wraps.push((dot..dot, format!("{path}\n{}", indent_at(s.text, dot))));
        path = "";
    }
    Ok(StepEdits::InPlace { head: format!("{KTLINT_STEP}.{base}{path}"), renames, arg_wraps })
}

/// Puts `io.github.hexay:ktrs:<version>` on the file's `buildscript` classpath, from `mavenCentral()`.
pub(crate) fn ensure_classpath(s: &Script, version: &str, edits: &mut Edits) {
    if s.text.contains(&format!("{JAR_GROUP}:{JAR_ARTIFACT}:")) {
        return;
    }
    let classpath = format!("classpath(\"{JAR_GROUP}:{JAR_ARTIFACT}:{version}\")");
    let Some(&bs) = s.child_blocks(None, "buildscript").first() else {
        let u = indent_unit(s.text);
        let block =
            format!("buildscript {{\n{u}repositories {{ mavenCentral() }}\n{u}dependencies {{ {classpath} }}\n}}\n");
        insert_at_top(s, &block, edits);
        return;
    };
    match s.child_blocks(Some(bs), "repositories").first() {
        Some(&r) if !(r..s.close_of(r)).any(|i| s.is_ident(i, "mavenCentral")) => {
            append_to_block(s, r, "mavenCentral()", edits)
        }
        Some(_) => {}
        None => append_to_block(s, bs, "repositories { mavenCentral() }", edits),
    }
    match s.child_blocks(Some(bs), "dependencies").first() {
        Some(&d) => append_to_block(s, d, &classpath, edits),
        None => append_to_block(s, bs, &format!("dependencies {{ {classpath} }}"), edits),
    }
}
