//! Walks a Gradle script's IR and records the tool configuration it performs.

use std::collections::HashMap;

use super::findings::Findings;
use super::steps;
use crate::findings::{KtfmtPartial, KtlintPartial, style};
use crate::ir::{Expr, Scope, Seg, Stmt};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Ctx {
    Top,
    Plugins,
    Deps,
    Ktfmt,
    Ktlint,
    Kotlinter,
    Spotless,
    /// `true`: `kotlinGradle {}`.
    Section(bool),
}

pub(crate) struct Interp<'a> {
    pub scope: &'a Scope<'a>,
    pub funs: HashMap<&'a str, &'a [Stmt]>,
    pub label: String,
    /// The module is the root project: `subprojects {}` doesn't apply to it.
    pub root_project: bool,
    pub out: Findings,
    /// `apply(from = ..)` scripts, for the caller to read.
    pub applied_from: Vec<String>,
    inlining: Vec<String>,
}

const RECEIVERS: [&str; 4] = ["this", "it", "project", "target"];

pub(crate) fn strip(segs: &[Seg]) -> &[Seg] {
    let n = segs.iter().take_while(|s| s.is_bare() && RECEIVERS.contains(&s.name.as_str())).count();
    &segs[n..]
}

fn ext_of_name(name: &str) -> Option<Ctx> {
    match name {
        "ktfmt" => Some(Ctx::Ktfmt),
        "ktlint" => Some(Ctx::Ktlint),
        "kotlinter" => Some(Ctx::Kotlinter),
        "spotless" => Some(Ctx::Spotless),
        _ => None,
    }
}

fn ext_of_type(name: &str) -> Option<Ctx> {
    match name.split(['<', ':']).next()?.rsplit('.').next()? {
        "KtfmtExtension" => Some(Ctx::Ktfmt),
        "KtlintExtension" => Some(Ctx::Ktlint),
        "KotlinterExtension" => Some(Ctx::Kotlinter),
        "SpotlessExtension" => Some(Ctx::Spotless),
        _ => None,
    }
}

/// `configure<T> {}`, `extensions.configure<T> {}`, `the<T>().apply {}`, `configure(T::class.java) {}`.
fn typed_ext_block(segs: &[Seg]) -> Option<(Ctx, &[Stmt])> {
    for (i, seg) in segs.iter().enumerate() {
        if !matches!(seg.name.as_str(), "configure" | "getByType" | "findByType" | "the" | "getByName") {
            continue;
        }
        let from_arg = || match seg.arg(0)? {
            Expr::Path(p) => ext_of_type(&p.first()?.name),
            _ => None,
        };
        let Some(ctx) = seg.type_args.first().and_then(|t| ext_of_type(t)).or_else(from_arg) else { continue };
        let block = seg.block.as_deref().or_else(|| segs.get(i + 1)?.block.as_deref())?;
        return Some((ctx, block));
    }
    None
}

impl<'a> Interp<'a> {
    pub(crate) fn new(scope: &'a Scope<'a>, funs: HashMap<&'a str, &'a [Stmt]>, label: String) -> Interp<'a> {
        Interp {
            scope,
            funs,
            label,
            root_project: false,
            out: Findings::default(),
            applied_from: Vec::new(),
            inlining: Vec::new(),
        }
    }

    pub(crate) fn note(&mut self, what: impl AsRef<str>) {
        let note = format!("{}: {}", self.label, what.as_ref());
        if !self.out.notes.contains(&note) {
            self.out.notes.push(note);
        }
    }

    pub(crate) fn text(&self, e: &Expr) -> Option<String> {
        self.scope.str(e).or_else(|| match e {
            Expr::Path(segs) => segs.last().map(|s| s.name.clone()),
            _ => None,
        })
    }

    pub(crate) fn stmts(&mut self, stmts: &[Stmt], ctx: Ctx) {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(Expr::Path(segs)) => self.path(strip(segs), ctx),
                Stmt::Assign(Expr::Path(lhs), rhs) => self.assign(strip(lhs), rhs, ctx),
                _ => {}
            }
        }
    }

    fn path(&mut self, segs: &[Seg], ctx: Ctx) {
        if segs.is_empty() {
            return;
        }
        let handled = match ctx {
            Ctx::Top => self.top(segs),
            Ctx::Plugins => self.plugin_decl(segs),
            Ctx::Deps => self.dependency(segs),
            Ctx::Ktfmt | Ctx::Ktlint | Ctx::Kotlinter => self.ext_call(segs, ctx),
            Ctx::Spotless => match (segs[0].name.as_str(), &segs[0].block) {
                ("kotlin", Some(b)) => self.enter(b, Ctx::Section(false)),
                ("kotlinGradle", Some(b)) => self.enter(b, Ctx::Section(true)),
                _ => false,
            },
            Ctx::Section(gradle) => steps::spotless_step(self, segs, gradle),
        };
        if !handled {
            for block in segs.iter().filter_map(|s| s.block.as_deref()) {
                self.stmts(block, ctx);
            }
        }
    }

    fn enter(&mut self, block: &[Stmt], ctx: Ctx) -> bool {
        self.stmts(block, ctx);
        true
    }

    fn mark_ext(&mut self, ctx: Ctx) {
        let (slot, name) = match ctx {
            Ctx::Ktfmt => {
                self.out.ktfmt.get_or_insert_with(KtfmtPartial::default);
                return self.note("ktfmt { }");
            }
            Ctx::Ktlint => (&mut self.out.ktlint, "ktlint { }"),
            Ctx::Kotlinter => (&mut self.out.kotlinter, "kotlinter { }"),
            _ => return,
        };
        slot.get_or_insert_with(KtlintPartial::default);
        self.note(name);
    }

    fn top(&mut self, segs: &[Seg]) -> bool {
        self.javaexec(segs, None);
        let first = &segs[0];
        let no_args = first.args().is_empty();
        if let (Some(block), true) = (&first.block, no_args) {
            match first.name.as_str() {
                "plugins" => return self.enter(block, Ctx::Plugins),
                "dependencies" => return self.enter(block, Ctx::Deps),
                "buildscript" => return true,
                "subprojects" if self.root_project => return true,
                name => {
                    if let Some(ctx) = ext_of_name(name) {
                        self.mark_ext(ctx);
                        return self.enter(block, ctx);
                    }
                }
            }
        }
        if first.is_bare()
            && segs.len() > 1
            && let Some(ctx) = ext_of_name(&first.name)
        {
            self.mark_ext(ctx);
            self.path(&segs[1..], ctx);
            return true;
        }
        if let Some((ctx, block)) = typed_ext_block(segs) {
            self.mark_ext(ctx);
            return self.enter(block, ctx);
        }
        if self.apply_plugin(segs) {
            return true;
        }
        if let [call] = segs
            && call.args.is_some()
            && call.block.is_none()
            && !self.inlining.contains(&call.name)
            && let Some(body) = self.funs.get(call.name.as_str()).copied()
        {
            self.inlining.push(call.name.clone());
            self.stmts(body, Ctx::Top);
            self.inlining.pop();
            return true;
        }
        false
    }

    /// Inside `ktfmt {}`, `ktlint {}` or `kotlinter {}`: `prop.set(v)`, `prop(v)`, `styleFn()`, map `put`.
    fn ext_call(&mut self, segs: &[Seg], ctx: Ctx) -> bool {
        match segs {
            [f] if ctx == Ctx::Ktfmt && f.args.is_some() && style(&f.name).is_some() => {
                self.out.ktfmt.get_or_insert_with(Default::default).style = style(&f.name);
                self.note(format!("ktfmt {}()", f.name));
                true
            }
            [p, s] if matches!(s.name.as_str(), "set" | "convention" | "put" | "putAll") => {
                let value = if s.name == "put" { s.arg(1) } else { s.arg(0) };
                if let (Some(v), true) = (value, s.name == "put") {
                    let key = s.arg(0).and_then(|k| self.scope.str(k)).unwrap_or_default();
                    let pair = Expr::Pair(Box::new(Expr::Str(key)), Box::new(v.clone()));
                    return self.option(ctx, &p.name, &pair);
                }
                value.is_some_and(|v| self.option(ctx, &p.name, v))
            }
            [p] if p.block.is_none() && p.args().len() == 1 => {
                let v = p.args()[0].value.clone();
                self.option(ctx, &p.name, &v)
            }
            _ => false,
        }
    }

    fn assign(&mut self, lhs: &[Seg], rhs: &Expr, ctx: Ctx) {
        match (ctx, lhs) {
            (Ctx::Ktfmt | Ctx::Ktlint | Ctx::Kotlinter, [p]) => {
                self.option(ctx, &p.name, rhs);
            }
            (Ctx::Top, [_]) => self.javaexec(lhs, Some(rhs)),
            (Ctx::Top, [ext, p]) if ext.is_bare() => {
                if let Some(ctx @ (Ctx::Ktfmt | Ctx::Ktlint | Ctx::Kotlinter)) = ext_of_name(&ext.name) {
                    self.mark_ext(ctx);
                    self.option(ctx, &p.name, rhs);
                }
            }
            _ => {}
        }
    }

    fn option(&mut self, ctx: Ctx, name: &str, value: &Expr) -> bool {
        if ctx == Ctx::Ktfmt {
            let Some(text) = self.text(value) else { return false };
            let set = self.out.ktfmt.get_or_insert_with(Default::default).set(name, &text);
            if set {
                self.note(format!("ktfmt {name} = {text}"));
            }
            return set;
        }
        let (tool, mut p) = match ctx {
            Ctx::Ktlint => ("ktlint", self.out.ktlint.take().unwrap_or_default()),
            _ => ("kotlinter", self.out.kotlinter.take().unwrap_or_default()),
        };
        let set = steps::ktlint_option(self, &mut p, name, value);
        if set {
            self.note(format!("{tool} {name}"));
        }
        match ctx {
            Ctx::Ktlint => self.out.ktlint = Some(p),
            _ => self.out.kotlinter = Some(p),
        }
        set
    }
}
