//! `psi.delete()` / `CodeEditUtil.removeChild` on every case of `tests/data/edit_cases.tsv`, dump for dump
//! against the ktlint fat jar (`tools/ktlint-oracle/EditOracle.java`, which has the regeneration command).

mod common;

use common::ast;
use ktrs_ast::{code_edit_util, psi};

fn run(name: &str, op: &str, kind: &str, nth: usize, text: &str) -> String {
    let mut a = ast(text);
    let node = a.preorder(a.root()).filter(|&n| format!("{:?}", a.element_type(n)) == kind).nth(nth).unwrap();
    match op {
        "delete" => psi::delete(&mut a, node),
        _ => {
            let parent = a.tree_parent(node).unwrap();
            code_edit_util::remove_child(&mut a, parent, node);
        }
    }
    format!("=== {name}\n{}\n", a.psi_to_string(a.root(), "File.kt"))
}

#[test]
fn edits_match_the_jvm() {
    let cases = include_str!("data/edit_cases.tsv");
    let expected = include_str!("data/edit_cases.jvm.txt").replace("\r\n", "\n");
    let mut expected_cases = expected.split("=== ").filter(|s| !s.is_empty());
    let mut failures = Vec::new();
    for line in cases.lines().filter(|l| !l.trim().is_empty()) {
        let c: Vec<&str> = line.splitn(5, '\t').collect();
        let actual = run(c[0], c[1], c[2], c[3].parse().unwrap(), &c[4].replace("\\n", "\n"));
        let want = format!("=== {}", expected_cases.next().expect("fewer JVM cases than inputs"));
        if actual != want {
            failures.push(format!("{}\n--- jvm\n{want}--- rust\n{actual}", c[0]));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
