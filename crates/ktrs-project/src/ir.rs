//! The call IR both build-script front ends lower to, and the evaluation of its values.
//! `a.b("x").c { d = 1 }` is one `Path` of three `Seg`s; Groovy commands (`id 'x' version 'y'`) and Kotlin
//! infix calls (`id("x") version "y"`) become the same chained segments.

use std::collections::HashMap;

use crate::catalog::Catalog;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
    Str(String),
    Template(Vec<Part>),
    Int(i64),
    Bool(bool),
    Path(Vec<Seg>),
    /// `a to b`, Groovy `a: b`.
    Pair(Box<Expr>, Box<Expr>),
    /// Groovy `[a, b]` / `[k: v]` (a list of `Pair`s).
    List(Vec<Expr>),
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Part {
    Lit(String),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Seg {
    pub name: String,
    pub type_args: Vec<String>,
    /// `None`: a property access, not a call.
    pub args: Option<Vec<Arg>>,
    pub block: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Arg {
    pub name: Option<String>,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stmt {
    Expr(Expr),
    Assign(Expr, Expr),
    Val(String, Expr),
    Fun(String, Vec<Stmt>),
    Class(String, Vec<Stmt>),
}

impl Seg {
    pub(crate) fn named(name: impl Into<String>) -> Seg {
        Seg { name: name.into(), ..Seg::default() }
    }

    pub(crate) fn is_bare(&self) -> bool {
        self.args.is_none() && self.block.is_none()
    }

    pub(crate) fn args(&self) -> &[Arg] {
        self.args.as_deref().unwrap_or_default()
    }

    pub(crate) fn arg(&self, i: usize) -> Option<&Expr> {
        self.args().iter().filter(|a| a.name.is_none()).nth(i).map(|a| &a.value)
    }

    pub(crate) fn named_arg(&self, name: &str) -> Option<&Expr> {
        self.args().iter().find(|a| a.name.as_deref() == Some(name)).map(|a| &a.value)
    }
}

/// Values a file can name: its `val`s / `def`s / `ext` properties, `gradle.properties`, the `libs` catalog.
pub(crate) struct Scope<'a> {
    pub vals: HashMap<String, Expr>,
    pub props: &'a HashMap<String, String>,
    pub catalog: &'a Catalog,
}

const PASS_THROUGH: [&str; 5] = ["get", "toString", "orNull", "trim", "asProvider"];
const MAX_DEPTH: u32 = 8;

impl<'a> Scope<'a> {
    pub(crate) fn new(stmts: &[Stmt], props: &'a HashMap<String, String>, catalog: &'a Catalog) -> Scope<'a> {
        let mut vals = HashMap::new();
        collect_vals(stmts, &mut vals);
        Scope { vals, props, catalog }
    }

    pub(crate) fn str(&self, e: &Expr) -> Option<String> {
        self.str_at(e, 0)
    }

    fn str_at(&self, e: &Expr, depth: u32) -> Option<String> {
        if depth > MAX_DEPTH {
            return None;
        }
        match e {
            Expr::Str(s) => Some(s.clone()),
            Expr::Int(i) => Some(i.to_string()),
            Expr::Bool(b) => Some(b.to_string()),
            Expr::Template(parts) => parts
                .iter()
                .map(|p| match p {
                    Part::Lit(s) => Some(s.clone()),
                    Part::Expr(e) => self.str_at(e, depth + 1),
                })
                .collect(),
            Expr::Path(segs) => self.path_str(segs, depth),
            _ => None,
        }
    }

    fn path_str(&self, segs: &[Seg], depth: u32) -> Option<String> {
        if let Some(v) = self.catalog_ref(segs) {
            return v;
        }
        if let Some(seg) = segs.iter().find(|s| matches!(s.name.as_str(), "property" | "findProperty" | "gradleProperty"))
        {
            return self.props.get(&self.str_at(seg.arg(0)?, depth + 1)?).cloned();
        }
        let (first, rest) = segs.split_first()?;
        if !first.is_bare() || !rest.iter().all(|s| PASS_THROUGH.contains(&s.name.as_str())) {
            return None;
        }
        match self.vals.get(&first.name) {
            Some(v) => self.str_at(v, depth + 1),
            None => self.props.get(&first.name).cloned(),
        }
    }

    /// `Some(value)` when `segs` is a `libs` accessor (`libs.versions.x`, `libs.plugins.x.get().pluginId`,
    /// `libs.findVersion("x").get()`, a library alias -> `group:name:version`).
    fn catalog_ref(&self, segs: &[Seg]) -> Option<Option<String>> {
        let (first, rest) = segs.split_first()?;
        if first.name != "libs" || !first.is_bare() {
            return None;
        }
        let next = rest.first()?;
        let found = match next.name.as_str() {
            "findVersion" | "findLibrary" | "findPlugin" => {
                let alias = self.str(next.arg(0)?)?;
                match next.name.as_str() {
                    "findVersion" => self.catalog.version(&alias),
                    "findLibrary" => self.catalog.library(&alias),
                    _ => self.catalog.plugin(&alias),
                }
            }
            _ => {
                let names: Vec<&str> = rest.iter().take_while(|s| s.is_bare()).map(|s| s.name.as_str()).collect();
                match names.split_first()? {
                    (&"versions", alias) => self.catalog.version(&alias.join(".")),
                    (&"plugins", alias) => self.catalog.plugin(&alias.join(".")),
                    (&"bundles", _) => None,
                    _ => self.catalog.library(&names.join(".")),
                }
            }
        };
        Some(found.map(str::to_string))
    }

    pub(crate) fn bool(&self, e: &Expr) -> Option<bool> {
        match e {
            Expr::Bool(b) => Some(*b),
            _ => self.str(e)?.trim().parse().ok(),
        }
    }

    /// `listOf(..)`, `setOf(..)`, Groovy `[..]`, or a single value.
    pub(crate) fn strings(&self, e: &Expr) -> Vec<String> {
        match e {
            Expr::List(items) => items.iter().filter_map(|i| self.str(i)).collect(),
            Expr::Path(segs) if segs.len() == 1 && is_collection_builder(&segs[0].name) => {
                segs[0].args().iter().filter_map(|a| self.str(&a.value)).collect()
            }
            _ => self.str(e).into_iter().collect(),
        }
    }

    /// `mapOf("k" to "v")`, Groovy `["k": "v"]`; values rendered as strings.
    pub(crate) fn map(&self, e: &Expr) -> Vec<(String, String)> {
        let items: Vec<&Expr> = match e {
            Expr::List(items) => items.iter().collect(),
            Expr::Path(segs) if segs.len() == 1 && is_collection_builder(&segs[0].name) => {
                segs[0].args().iter().map(|a| &a.value).collect()
            }
            Expr::Pair(..) => vec![e],
            _ => Vec::new(),
        };
        items
            .into_iter()
            .filter_map(|i| match i {
                Expr::Pair(k, v) => Some((self.str(k)?, self.str(v)?)),
                _ => None,
            })
            .collect()
    }
}

/// A value as written, for unresolvable dependency notations (`projects.rules`, `files("x.jar")`).
pub(crate) fn render(e: &Expr) -> String {
    match e {
        Expr::Str(s) => s.clone(),
        Expr::Int(i) => i.to_string(),
        Expr::Bool(b) => b.to_string(),
        Expr::Path(segs) => segs
            .iter()
            .map(|s| match &s.args {
                Some(args) => {
                    format!("{}({})", s.name, args.iter().map(|a| render(&a.value)).collect::<Vec<_>>().join(", "))
                }
                None => s.name.clone(),
            })
            .collect::<Vec<_>>()
            .join("."),
        _ => "?".into(),
    }
}

fn is_collection_builder(name: &str) -> bool {
    matches!(
        name,
        "listOf" | "setOf" | "arrayOf" | "mutableListOf" | "mutableSetOf" | "mapOf" | "mutableMapOf" | "hashMapOf"
    )
}

fn collect_vals(stmts: &[Stmt], vals: &mut HashMap<String, Expr>) {
    for stmt in stmts {
        match stmt {
            Stmt::Val(name, e) => {
                vals.insert(name.clone(), e.clone());
            }
            Stmt::Assign(Expr::Path(segs), e) => {
                let names: Vec<&str> = segs.iter().map(|s| s.name.as_str()).collect();
                if let ["ext" | "extra", name] | ["project" | "rootProject", "ext" | "extra", name] = names[..] {
                    vals.insert(name.to_string(), e.clone());
                }
            }
            Stmt::Expr(Expr::Path(segs)) if segs.len() == 1 && segs[0].name == "ext" => {
                for s in segs[0].block.iter().flatten() {
                    if let Stmt::Assign(Expr::Path(lhs), e) = s
                        && let [seg] = &lhs[..]
                    {
                        vals.insert(seg.name.clone(), e.clone());
                    }
                }
            }
            // `buildscript { ext { .. } }`, `allprojects { val x = .. }`.
            Stmt::Expr(Expr::Path(segs)) => {
                for block in segs.iter().filter_map(|s| s.block.as_ref()) {
                    collect_vals(block, vals);
                }
            }
            _ => {}
        }
    }
}
