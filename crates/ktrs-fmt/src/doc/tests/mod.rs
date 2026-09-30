//! Doc engine tests. `cases.txt` holds op scripts over Kotlin inputs; `cases.expected` is the
//! output of google-java-format's real engine (via ktfmt 0.64's jar) on the same scripts;
//! regenerate with `tools/ktfmt-oracle/engine/engine-oracle.sh cases` (`DocScript.java`).
//! In `cases.txt`, `⎵` stands for a trailing space.

mod script;
mod units;

fn sections(text: &str) -> Vec<(&str, &str)> {
    let text = text.strip_prefix("=== ").expect("starts with a section");
    text.split("\n=== ")
        .map(|section| section.split_once('\n').unwrap_or((section, "")))
        .collect()
}

#[test]
fn cases_match_gjf() {
    let cases = include_str!("cases.txt").replace("\r\n", "\n");
    let expected = include_str!("cases.expected").replace("\r\n", "\n");
    // The JVM harness terminates every output with "\n"; all but the last become separators.
    let expected: std::collections::HashMap<&str, &str> =
        sections(expected.strip_suffix('\n').unwrap())
            .into_iter()
            .collect();
    let mut failures = Vec::new();
    for (name, body) in sections(&cases) {
        let (header, code) = body.split_once('\n').unwrap_or((body, ""));
        let (width, script) = header.split_once(' ').unwrap();
        let code = code.replace('⎵', " ");
        let actual = script::run(&code, width.parse().unwrap(), script);
        let want = expected[name];
        if actual != want {
            failures.push(format!("--- {name}\nexpected:\n{want}\nactual:\n{actual}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
