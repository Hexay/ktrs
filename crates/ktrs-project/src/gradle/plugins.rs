//! Plugin application (`plugins {}`, `apply(..)` forms) and the `ktlint` / `ktlintRuleset` dependencies.

use super::findings::{is_tool_plugin, plugin_of_type};
use super::interpret::Interp;
use crate::ir::{Expr, Seg, render};

impl Interp<'_> {
    pub(crate) fn add_plugin(&mut self, id: String) {
        if is_tool_plugin(&id) {
            self.note(format!("plugin {id}"));
        }
        if !self.out.plugins.contains(&id) {
            self.out.plugins.push(id);
        }
    }

    /// `apply(plugin = ..)`, `apply plugin: ..`, `pluginManager.apply(..)`, `apply<T>()`, `apply(from = ..)`.
    pub(crate) fn apply_plugin(&mut self, segs: &[Seg]) -> bool {
        for (i, seg) in segs.iter().enumerate() {
            if seg.name != "apply" || seg.args.is_none() {
                continue;
            }
            if let Some(from) = seg.named_arg("from") {
                if let Some(path) = self.scope.str(from) {
                    self.applied_from.push(path);
                }
                return true;
            }
            if let Some(t) = seg.type_args.first() {
                if let Some(id) = plugin_of_type(t) {
                    self.add_plugin(id.to_string());
                }
                return true;
            }
            let receiver_ok = i == 0 || matches!(segs[i - 1].name.as_str(), "pluginManager" | "plugins");
            let Some(arg) = seg.named_arg("plugin").or_else(|| seg.arg(0).filter(|_| receiver_ok)) else {
                continue;
            };
            let id = match arg {
                Expr::Path(p) if p.len() == 1 => plugin_of_type(&p[0].name).map(str::to_string),
                _ => self.scope.str(arg),
            };
            if let Some(id) = id.filter(|id| !id.contains('/') && !id.ends_with(".gradle") && !id.ends_with(".kts")) {
                self.add_plugin(id);
            }
            return true;
        }
        false
    }

    /// A `plugins {}` entry: `id(..)`, `alias(..)`; `apply false` only declares.
    pub(crate) fn plugin_decl(&mut self, segs: &[Seg]) -> bool {
        let first = &segs[0];
        if !matches!(first.name.as_str(), "id" | "alias") {
            return true;
        }
        let Some(id) = first.arg(0).and_then(|a| self.scope.str(a)) else { return true };
        let declared_only = segs[1..].iter().any(|s| s.name == "apply" && s.arg(0) == Some(&Expr::Bool(false)));
        if declared_only {
            if is_tool_plugin(&id) {
                self.note(format!("plugin {id} declared with apply false"));
            }
        } else {
            self.add_plugin(id);
        }
        true
    }

    /// The ktlint JavaExec recipe: `mainClass = "..ktlint..Main"` (or `.set(..)`), `args("**/*.kt")`.
    pub(crate) fn javaexec(&mut self, segs: &[Seg], assigned: Option<&Expr>) {
        let first = &segs[0];
        let main = match (first.name.as_str(), assigned, segs.get(1)) {
            ("mainClass" | "main", Some(v), _) => Some(v),
            ("mainClass" | "main", None, Some(s)) if s.name == "set" => s.arg(0),
            _ => None,
        };
        if main.and_then(|v| self.scope.str(v)).is_some_and(|m| m.contains("ktlint") && m.ends_with("Main")) {
            self.out.ktlint_main_class = true;
        }
        let wide = |a: &crate::ir::Arg| self.scope.str(&a.value).is_some_and(|s| s.contains("**"));
        if first.name == "args" && first.args().iter().any(wide) {
            self.out.wide_args = true;
        }
    }

    pub(crate) fn dependency(&mut self, segs: &[Seg]) -> bool {
        let first = &segs[0];
        let (config, value) = match (first.name.as_str(), first.arg(0), first.arg(1)) {
            ("add", Some(c), Some(v)) => (self.scope.str(c).unwrap_or_default(), v),
            (name, Some(v), _) => (name.to_string(), v),
            _ => return false,
        };
        if config != "ktlint" && config != "ktlintRuleset" {
            return false;
        }
        let coords = self.scope.str(value).unwrap_or_else(|| render(value));
        self.note(format!("{config}({coords})"));
        let list = if config == "ktlint" { &mut self.out.ktlint_deps } else { &mut self.out.ruleset_deps };
        list.push(coords);
        true
    }
}
