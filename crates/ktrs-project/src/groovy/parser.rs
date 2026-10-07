//! A tolerant parser for the build-script subset of Groovy: calls with parentheses, closures, command
//! expressions (`id 'x' version 'y'`, `apply plugin: 'x'`), assignments, `def` locals, lists and maps.
//! Anything else (operators, control flow conditions, class bodies) is skipped to the end of the statement.

use super::lexer::{Tok, lex};
use crate::ir::{Arg, Expr, Seg, Stmt};

pub(crate) fn parse(text: &str) -> Vec<Stmt> {
    Parser { toks: lex(text), pos: 0 }.block(false)
}

pub(super) fn parse_expr(text: &str) -> Expr {
    Parser { toks: lex(text), pos: 0 }.expr(false)
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

const MODIFIERS: [&str; 6] = ["def", "final", "private", "static", "public", "var"];

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn peek_at(&self, n: usize) -> Option<&Tok> {
        self.toks.get(self.pos + n)
    }

    fn at(&self, p: &str) -> bool {
        matches!(self.peek(), Some(Tok::Punct(q)) if *q == p)
    }

    fn eat(&mut self, p: &str) -> bool {
        let found = self.at(p);
        if found {
            self.pos += 1;
        }
        found
    }

    fn skip_newlines(&mut self) {
        while self.peek() == Some(&Tok::Newline) {
            self.pos += 1;
        }
    }

    fn ident(&self, n: usize) -> Option<&str> {
        if let Some(Tok::Ident(s)) = self.peek_at(n) { Some(s) } else { None }
    }

    fn block(&mut self, nested: bool) -> Vec<Stmt> {
        let mut out = Vec::new();
        if nested {
            self.skip_closure_params();
        }
        loop {
            while self.peek() == Some(&Tok::Newline) || self.at(";") {
                self.pos += 1;
            }
            match self.peek() {
                None => break,
                Some(Tok::Punct("}")) => {
                    self.pos += 1;
                    if nested {
                        break;
                    }
                    continue;
                }
                _ => {}
            }
            let start = self.pos;
            if let Some(stmt) = self.statement() {
                out.push(stmt);
            }
            self.skip_statement_rest();
            if self.pos == start {
                self.pos += 1;
            }
        }
        out
    }

    /// `{ a, b -> ...`
    fn skip_closure_params(&mut self) {
        let mut n = 0;
        while matches!(self.peek_at(n), Some(Tok::Ident(_)) | Some(Tok::Punct(","))) {
            n += 1;
        }
        if n > 0 && matches!(self.peek_at(n), Some(Tok::Punct("->"))) {
            self.pos += n + 1;
        }
    }

    fn statement(&mut self) -> Option<Stmt> {
        match self.ident(0) {
            Some("import" | "package") => return None,
            Some("return") => self.pos += 1,
            _ => {}
        }
        let mut declared = false;
        while self.ident(0).is_some_and(|i| MODIFIERS.contains(&i)) {
            self.pos += 1;
            declared = true;
        }
        // `def x = ..`, `String x = ..`
        if self.ident(0).is_some() && self.ident(1).is_some() && self.peek_at(2) == Some(&Tok::Punct("=")) {
            self.pos += 1;
            declared = true;
        }
        if declared
            && let Some(name) = self.ident(0).map(str::to_string)
            && self.peek_at(1) == Some(&Tok::Punct("="))
        {
            self.pos += 2;
            return Some(Stmt::Val(name, self.expr(true)));
        }
        let lhs = self.expr(true);
        if self.eat("=") {
            return Some(Stmt::Assign(lhs, self.expr(true)));
        }
        Some(Stmt::Expr(lhs))
    }

    fn skip_statement_rest(&mut self) {
        let mut depth = 0usize;
        while let Some(tok) = self.peek() {
            match tok {
                Tok::Newline | Tok::Punct(";") if depth == 0 => return,
                Tok::Punct("}") if depth == 0 => return,
                Tok::Punct("(" | "[" | "{") => depth += 1,
                Tok::Punct(")" | "]" | "}") => depth = depth.saturating_sub(1),
                _ => {}
            }
            self.pos += 1;
        }
    }

    fn expr(&mut self, command: bool) -> Expr {
        let first = match self.peek().cloned() {
            Some(Tok::Str(e)) => {
                self.pos += 1;
                return e;
            }
            Some(Tok::Num(n)) => {
                self.pos += 1;
                return n.trim_end_matches(|c: char| c.is_ascii_alphabetic()).parse().map_or(Expr::Other, Expr::Int);
            }
            Some(Tok::Ident(name)) => {
                self.pos += 1;
                match name.as_str() {
                    "true" => return Expr::Bool(true),
                    "false" => return Expr::Bool(false),
                    "null" => return Expr::Other,
                    _ => Seg::named(name),
                }
            }
            Some(Tok::Punct("[")) => {
                self.pos += 1;
                return self.list();
            }
            Some(Tok::Punct("(")) => {
                self.pos += 1;
                self.skip_newlines();
                let inner = self.expr(false);
                self.skip_to_close(")");
                return inner;
            }
            Some(_) => {
                self.pos += 1;
                return Expr::Other;
            }
            None => return Expr::Other,
        };
        self.postfix(vec![first], command)
    }

    fn postfix(&mut self, mut segs: Vec<Seg>, command: bool) -> Expr {
        loop {
            let last = segs.last_mut().expect("non-empty");
            if self.at("(") && last.args.is_none() {
                self.pos += 1;
                last.args = Some(self.args(")"));
            } else if self.at("{") && last.block.is_none() {
                self.pos += 1;
                last.block = Some(self.block(true));
            } else if (self.at(".") || self.at("?.") || self.at("*.")) && self.member_name(1).is_some() {
                let name = self.member_name(1).unwrap_or_default();
                self.pos += 2;
                segs.push(Seg::named(name));
            } else if command && last.is_bare() && self.value_starts(0) {
                last.args = Some(self.command_args());
            } else if command
                && !last.is_bare()
                && self.ident(0).is_some()
                && (self.value_starts(1) || self.peek_at(1) == Some(&Tok::Punct("{")))
            {
                // `id 'x' version 'y'`, `} else {`
                let name = self.ident(0).unwrap_or_default().to_string();
                self.pos += 1;
                segs.push(Seg::named(name));
            } else {
                return Expr::Path(segs);
            }
        }
    }

    fn member_name(&self, n: usize) -> Option<String> {
        match self.peek_at(n)? {
            Tok::Ident(s) => Some(s.clone()),
            Tok::Str(Expr::Str(s)) => Some(s.clone()),
            _ => None,
        }
    }

    fn value_starts(&self, n: usize) -> bool {
        match self.peek_at(n) {
            Some(Tok::Str(_) | Tok::Num(_) | Tok::Punct("[")) => true,
            Some(Tok::Ident(s)) => !matches!(s.as_str(), "in" | "instanceof" | "as"),
            _ => false,
        }
    }

    fn arg(&mut self) -> Arg {
        let named = matches!(self.peek(), Some(Tok::Ident(_) | Tok::Str(Expr::Str(_))))
            && self.peek_at(1) == Some(&Tok::Punct(":"));
        let name = if named {
            let name = self.member_name(0);
            self.pos += 2;
            self.skip_newlines();
            name
        } else {
            None
        };
        Arg { name, value: self.expr(false) }
    }

    fn command_args(&mut self) -> Vec<Arg> {
        let mut args = vec![self.arg()];
        while self.eat(",") {
            self.skip_newlines();
            args.push(self.arg());
        }
        args
    }

    fn args(&mut self, close: &str) -> Vec<Arg> {
        let mut args = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek().is_none() || self.eat(close) {
                return args;
            }
            let start = self.pos;
            args.push(self.arg());
            self.skip_newlines();
            if !self.eat(",") {
                self.skip_to_close(close);
                return args;
            }
            if self.pos == start {
                self.pos += 1;
            }
        }
    }

    /// `[a, b]` or `[k: v]` (`[:]` is an empty map); the `[` is consumed.
    fn list(&mut self) -> Expr {
        if self.eat(":") {
            self.skip_to_close("]");
            return Expr::List(Vec::new());
        }
        let items = self
            .args("]")
            .into_iter()
            .map(|a| match a.name {
                Some(k) => Expr::Pair(Box::new(Expr::Str(k)), Box::new(a.value)),
                None => a.value,
            })
            .collect();
        Expr::List(items)
    }

    /// Skips past the matching `close`, balancing brackets.
    fn skip_to_close(&mut self, close: &str) {
        let mut depth = 0usize;
        while let Some(tok) = self.peek().cloned() {
            self.pos += 1;
            match tok {
                Tok::Punct(p) if p == close && depth == 0 => return,
                Tok::Punct("(" | "[" | "{") => depth += 1,
                Tok::Punct(")" | "]" | "}") => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
    }
}
