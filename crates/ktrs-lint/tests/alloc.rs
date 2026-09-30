//! Go/no-go (c) of research/11 "Prototype first": navigation allocates nothing, and a 3-rule traversal
//! allocates nothing per node (only its children stack grows, O(log width) times).

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

use ktrs_ast::{Ast, NodeId};
use ktrs_lint::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use ktrs_lint::engine::{FORMATTER_TAGS_ENABLED_PROPERTY, SuppressionLocator, execute_rules};
use ktrs_lint::rules::standard_rule_provider;
use ktrs_lint::{AstNodeExtension, AutocorrectDecision, EditorConfig, RuleV2};
use ktrs_parser::{FileKind, parse_file};

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.with(Cell::get) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if COUNTING.with(Cell::get) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn allocations_during(f: impl FnOnce()) -> usize {
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    COUNTING.with(|c| c.set(true));
    f();
    COUNTING.with(|c| c.set(false));
    ALLOCATIONS.load(Ordering::Relaxed) - before
}

/// Clean for all three rules: every branch braced, commas spaced, no semicolons.
fn clean_source(classes: usize) -> String {
    (0..classes)
        .map(|i| {
            format!(
                "/** Doc {i}. */\nclass C{i}(private val a: Int, val b: String) {{\n    fun f(x: Int, y: Int): Int {{\n        \
                 // compare\n        if (x > y) {{\n            return x\n        }} else {{\n            return y\n        }}\n    \
                 }}\n\n    val list = listOf(1, 2, 3).map {{ it * 2 }}\n}}\n\n"
            )
        })
        .collect()
}

fn preorder(ast: &Ast) -> Vec<NodeId> {
    let mut out = Vec::new();
    let mut stack = vec![ast.root()];
    while let Some(n) = stack.pop() {
        out.push(n);
        let mut children = Vec::new();
        ast.get_children(n, &mut children);
        stack.extend(children.into_iter().rev());
    }
    out
}

#[test]
fn navigation_and_traversal_do_not_allocate_per_node() {
    let text = clean_source(400);
    let mut ast = Ast::from_parse(&parse_file(&text, FileKind::Source));
    let nodes = preorder(&ast);
    assert!(nodes.len() > 50_000, "{} nodes", nodes.len());

    let mut sink = 0usize;
    let navigation = allocations_during(|| {
        for &n in &nodes {
            sink += ast.start_offset(n) + ast.text_length(n);
            sink += [ast.next_leaf(n), ast.prev_leaf(n), ast.prev_code_leaf(n), ast.next_code_leaf(n)].iter().flatten().count();
            sink += [ast.next_code_sibling(n), ast.prev_code_sibling(n), ast.find_parent_by_type(n, ktrs_syntax::SyntaxKind::CLASS_BODY)]
                .iter()
                .flatten()
                .count();
            sink += usize::from(ast.is_part_of_comment(n)) + usize::from(ast.is_part_of_string(n)) + usize::from(ast.text_contains(n, '\n'));
            sink += usize::from(ast.text_matches(n, ",")) + ast.leaves(n, false).take(3).count() + ast.children(n).count();
        }
    });
    assert_eq!(navigation, 0, "navigation allocated (checksum {sink})");

    let rules: Vec<Box<dyn RuleV2>> = ["comma-spacing", "multiline-if-else", "no-semi"]
        .iter()
        .map(|id| standard_rule_provider(id).unwrap().create_new_rule_instance())
        .collect();
    let config = EditorConfig::default().filter_by(&[
        PropertyRef::from(&*FORMATTER_TAGS_ENABLED_PROPERTY),
        PropertyRef::from(&*INDENT_SIZE_PROPERTY),
        PropertyRef::from(&*INDENT_STYLE_PROPERTY),
    ]);
    let mut suppression_locator = SuppressionLocator::new(&config);
    // The suppression hints are built once per text, not per node.
    suppression_locator.suppress(&ast, ast.root(), 0, &*rules[0]);
    let mut emits = 0;
    let traversal = allocations_during(|| {
        execute_rules(&mut ast, rules, &config, &mut suppression_locator, &mut |_, _, _, _| {
            emits += 1;
            AutocorrectDecision::NoAutocorrect
        })
        .unwrap()
    });
    assert_eq!(emits, 0, "the input is meant to be clean");
    println!("{} nodes: navigation {navigation} allocations, 3-rule traversal {traversal}", nodes.len());
    assert!(traversal <= 24, "traversal allocated {traversal} times over {} nodes", nodes.len());
}
