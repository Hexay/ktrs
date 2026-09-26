//! Prototype for research/06-tree-library.md: rowan green/red trees vs a flat preorder tree.
//! `cargo run -p ktrs_parser --release --example tree_bench [dir] [reps] [walks]`
//!
//! Each corpus file is parsed once and recorded as start/token/finish events. The events are then
//! replayed into (a) rowan, built exactly like `TreeSink` (manual children Vec, `GreenNode::new`,
//! 1024-slot token interner) and (b) a flat tree (preorder arrays, node = index). Per tree it times
//! build, `walks` visitor-style walks (child/sibling steps, cloned handles, token text reads, one
//! child-by-kind lookup per node, like the PSI accessors) and drop, best of `reps`, and counts
//! allocations per phase.

use std::alloc::{GlobalAlloc, Layout};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;

use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::{KotlinLanguage, SyntaxKind};
use rowan::{GreenNode, GreenNodeData, GreenToken, NodeOrToken};

struct Counting;
static ALLOCS: AtomicU64 = AtomicU64::new(0);
static MI: mimalloc::MiMalloc = mimalloc::MiMalloc;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        unsafe { MI.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { MI.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        unsafe { MI.realloc(p, l, n) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

#[derive(Clone, Copy)]
enum Ev {
    Start(u16),
    Token(u16, u32),
    Finish,
}

fn record(node: &GreenNodeData, out: &mut Vec<Ev>) {
    out.push(Ev::Start(node.kind().0));
    for child in node.children() {
        match child {
            NodeOrToken::Node(n) => record(n, out),
            NodeOrToken::Token(t) => out.push(Ev::Token(t.kind().0, t.text().len() as u32)),
        }
    }
    out.push(Ev::Finish);
}

// ---- rowan, built like ktrs_parser's TreeSink + interner ----

type Green = NodeOrToken<GreenNode, GreenToken>;

struct RowanSink {
    children: Vec<Green>,
    parents: Vec<(u16, usize)>,
    interner: Box<[Option<GreenToken>]>,
}

fn build_rowan(sink: &mut RowanSink, events: &[Ev], text: &str) -> GreenNode {
    let mut offset = 0usize;
    for &ev in events {
        match ev {
            Ev::Start(kind) => sink.parents.push((kind, sink.children.len())),
            Ev::Token(kind, len) => {
                let t = &text[offset..offset + len as usize];
                offset += len as usize;
                let kind = rowan::SyntaxKind(kind);
                let slot = &mut sink.interner[hash(kind, t.as_bytes()) & 1023];
                let token = match slot {
                    Some(tok) if tok.kind() == kind && tok.text() == t => tok.clone(),
                    _ => slot.insert(GreenToken::new(kind, t)).clone(),
                };
                sink.children.push(token.into());
            }
            Ev::Finish => {
                let (kind, first) = sink.parents.pop().unwrap();
                let node = GreenNode::new(rowan::SyntaxKind(kind), sink.children.drain(first..));
                sink.children.push(node.into());
            }
        }
    }
    sink.children.pop().and_then(NodeOrToken::into_node).unwrap()
}

fn hash(kind: rowan::SyntaxKind, bytes: &[u8]) -> usize {
    let mut h = kind.0 as u64;
    let mut mix = |w: u64| h = (h.rotate_left(5) ^ w).wrapping_mul(0x517c_c1b7_2722_0a95);
    let (words, rest) = bytes.as_chunks::<8>();
    for &w in words {
        mix(u64::from_le_bytes(w));
    }
    let mut tail = [0u8; 8];
    tail[..rest.len()].copy_from_slice(rest);
    mix(u64::from_le_bytes(tail) ^ (bytes.len() as u64) << 56);
    (h ^ h >> 29) as usize
}

type RNode = rowan::SyntaxNode<KotlinLanguage>;

fn walk_rowan(node: &RNode) -> u64 {
    let mut sum = node.kind() as u64;
    let probe = node.first_child_or_token_by_kind(&|k| k == SyntaxKind::IDENTIFIER);
    sum += probe.is_some() as u64;
    let mut child = node.first_child_or_token();
    while let Some(c) = child {
        match &c {
            NodeOrToken::Node(n) => sum += walk_rowan(n),
            NodeOrToken::Token(t) => sum += t.text().len() as u64,
        }
        child = c.next_sibling_or_token();
    }
    sum
}

// ---- flat preorder tree ----

const NONE: u32 = u32::MAX;
/// Tokens and childless nodes both span one preorder slot; the kind's top bit tells them apart.
const TOKEN_BIT: u16 = 0x8000;

struct Tree {
    text: Box<str>,
    kind: Vec<u16>,
    start: Vec<u32>,
    /// Preorder index one past the element's subtree.
    end: Vec<u32>,
    parent: Vec<u32>,
}

fn build_flat(events: &[Ev], text: &str) -> Tree {
    let n = events.iter().filter(|e| !matches!(e, Ev::Finish)).count();
    let mut t = Tree {
        text: text.into(),
        kind: Vec::with_capacity(n),
        start: Vec::with_capacity(n),
        end: Vec::with_capacity(n),
        parent: Vec::with_capacity(n),
    };
    let mut stack: Vec<u32> = Vec::new();
    let mut offset = 0u32;
    for &ev in events {
        let idx = t.kind.len() as u32;
        match ev {
            Ev::Start(kind) => {
                t.kind.push(kind);
                t.start.push(offset);
                t.end.push(NONE);
                t.parent.push(stack.last().copied().unwrap_or(NONE));
                stack.push(idx);
            }
            Ev::Token(kind, len) => {
                t.kind.push(kind | TOKEN_BIT);
                t.start.push(offset);
                t.end.push(idx + 1);
                t.parent.push(stack.last().copied().unwrap_or(NONE));
                offset += len;
            }
            Ev::Finish => {
                let open = stack.pop().unwrap();
                t.end[open as usize] = idx;
            }
        }
    }
    t
}

/// What `PsiElement` would become: a shared tree plus an index.
#[derive(Clone)]
struct FNode {
    tree: Rc<Tree>,
    i: u32,
}

impl FNode {
    fn at(&self, i: u32) -> FNode {
        FNode { tree: self.tree.clone(), i }
    }
    fn kind(&self) -> u16 {
        self.tree.kind[self.i as usize] & !TOKEN_BIT
    }
    fn is_token(&self) -> bool {
        self.tree.kind[self.i as usize] & TOKEN_BIT != 0
    }
    fn first_child(&self) -> Option<FNode> {
        let e = self.tree.end[self.i as usize];
        (e > self.i + 1).then(|| self.at(self.i + 1))
    }
    fn next_sibling(&self) -> Option<FNode> {
        let p = self.tree.parent[self.i as usize];
        let e = self.tree.end[self.i as usize];
        (p != NONE && e < self.tree.end[p as usize]).then(|| self.at(e))
    }
    fn text(&self) -> &str {
        let t = &self.tree;
        let s = t.start[self.i as usize] as usize;
        let e = t.start.get(t.end[self.i as usize] as usize).map_or(t.text.len(), |&x| x as usize);
        &t.text[s..e]
    }
    fn first_child_by_kind(&self, kind: u16) -> Option<FNode> {
        let mut c = self.first_child();
        while let Some(n) = c {
            if n.kind() == kind {
                return Some(n);
            }
            c = n.next_sibling();
        }
        None
    }
}

fn walk_flat(node: &FNode) -> u64 {
    let mut sum = node.kind() as u64;
    sum += node.first_child_by_kind(SyntaxKind::IDENTIFIER as u16).is_some() as u64;
    let mut child = node.first_child();
    while let Some(c) = child {
        if c.is_token() { sum += c.text().len() as u64 } else { sum += walk_flat(&c) }
        child = c.next_sibling();
    }
    sum
}

// ---- driver ----

#[derive(Default, Clone, Copy)]
struct Phase {
    secs: f64,
    allocs: u64,
}

fn measure<R>(reps: u32, mut f: impl FnMut() -> R, keep: &mut Option<R>) -> Phase {
    let mut best = Phase { secs: f64::INFINITY, allocs: 0 };
    for _ in 0..reps.max(1) {
        let a = ALLOCS.load(Relaxed);
        let t = Instant::now();
        let r = std::hint::black_box(f());
        best.secs = best.secs.min(t.elapsed().as_secs_f64());
        best.allocs = ALLOCS.load(Relaxed) - a;
        *keep = Some(r);
    }
    best
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = PathBuf::from(args.first().map_or("corpus", String::as_str));
    let reps: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let walks: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
    let mut files = Vec::new();
    collect(&dir, &mut files);
    files.sort();

    let mut rowan_sink = RowanSink { children: Vec::new(), parents: Vec::new(), interner: vec![None; 1024].into() };
    let [mut rb, mut rw, mut rd, mut fb, mut fw, mut fd] = [Phase::default(); 6];
    let (mut bytes, mut checksum_mismatch) = (0usize, 0usize);
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap_or_default().replace("\r\n", "\n");
        let parse = parse_file(&text, FileKind::from_file_name(&file.to_string_lossy()));
        let mut events = Vec::new();
        record(&parse.green, &mut events);
        drop(parse);
        bytes += text.len();

        let add = |acc: &mut Phase, p: Phase| {
            acc.secs += p.secs;
            acc.allocs += p.allocs;
        };
        let mut green = None;
        add(&mut rb, measure(reps, || build_rowan(&mut rowan_sink, &events, &text), &mut green));
        let root = RNode::new_root(green.take().unwrap());
        let mut sum_r = None;
        add(&mut rw, measure(reps, || (0..walks).map(|_| walk_rowan(&root)).sum::<u64>(), &mut sum_r));
        add(&mut rd, drop_timed(root));

        let mut tree = None;
        add(&mut fb, measure(reps, || build_flat(&events, &text), &mut tree));
        let root = FNode { tree: Rc::new(tree.take().unwrap()), i: 0 };
        let mut sum_f = None;
        add(&mut fw, measure(reps, || (0..walks).map(|_| walk_flat(&root)).sum::<u64>(), &mut sum_f));
        add(&mut fd, drop_timed(root));
        checksum_mismatch += (sum_r != sum_f) as usize;
    }

    let mb = bytes as f64 / 1e6;
    println!("{} files, {mb:.1} MB, walks per tree = {walks}, best of {reps}; walk checksum mismatches: {checksum_mismatch}", files.len());
    println!("{:<8} {:>22} {:>22} {:>22}", "", "build", format!("{walks} walks"), "drop");
    for (name, b, w, d) in [("rowan", rb, rw, rd), ("flat", fb, fw, fd)] {
        let cell = |p: Phase| format!("{:7.3}s {:>12} allocs", p.secs, p.allocs);
        println!("{name:<8} {:>22} {:>22} {:>22}", cell(b), cell(w), cell(d));
    }
    println!(
        "speedup  build+drop {:.1}x, walks {:.1}x",
        (rb.secs + rd.secs) / (fb.secs + fd.secs),
        rw.secs / fw.secs
    );
}

fn drop_timed<T>(value: T) -> Phase {
    let a = ALLOCS.load(Relaxed);
    let t = Instant::now();
    drop(value);
    Phase { secs: t.elapsed().as_secs_f64(), allocs: ALLOCS.load(Relaxed) - a }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "kt" || e == "kts") {
            out.push(path);
        }
    }
}
