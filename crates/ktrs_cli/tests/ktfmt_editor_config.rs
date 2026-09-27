//! Port of ktfmt's `cli/EditorConfigResolverTest.kt` (v0.64).

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{TempDir, write_text};
use ktrs_cli::ktfmt::editor_config_resolver::resolve_formatting_options;
use ktrs_fmt::{FormattingOptions, GOOGLE_FORMAT, TrailingCommaManagementStrategy};

const ONLY_ADD: TrailingCommaManagementStrategy = TrailingCommaManagementStrategy::OnlyAdd;

/// Upstream's `root`. It sits one level below a `root = true` .editorconfig so that no
/// .editorconfig above the system temp dir leaks in (the tests without their own root file).
struct Root {
    shield: TempDir,
}

impl Root {
    fn new() -> Self {
        let shield = TempDir::new("editor_config");
        write_text(&shield.path().join(".editorconfig"), "root = true\n");
        fs::create_dir_all(shield.path().join("root")).unwrap();
        Root { shield }
    }

    fn resolve(&self, relative: &str) -> PathBuf {
        // `File.resolve` normalizes separators to the platform's.
        relative.split('/').fold(self.shield.path().join("root"), |path, part| path.join(part))
    }

    /// `root.resolve(relative).writeText(text.trimIndent())`.
    fn write_conf(&self, relative: &str, lines: &[&str]) {
        let conf = self.resolve(relative);
        fs::create_dir_all(conf.parent().unwrap()).unwrap();
        write_text(&conf, &lines.join("\n"));
    }
}

fn resolve(file: &Path) -> FormattingOptions {
    resolve_formatting_options(file, &GOOGLE_FORMAT)
}

#[test]
fn resolves_base_properties_when_no_editorconfig_file() {
    let root = Root::new();
    let src = root.resolve("src/main/kotlin/Example.kt");
    fs::create_dir_all(src.parent().unwrap()).unwrap();
    write_text(&src, "");
    assert_eq!(resolve(&src), GOOGLE_FORMAT);
}

#[test]
fn resolves_base_properties_editorconfig_file_doesnt_match() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.c]", "max_line_length = 80"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), GOOGLE_FORMAT);
}

#[test]
fn overrides_max_width_based_on_editorconfig_max_line_length() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.kt]", "max_line_length = 80"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), FormattingOptions { max_width: 80, ..GOOGLE_FORMAT });
}

#[test]
fn doesnt_override_max_width_based_when_editorconfig_max_line_length_off() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.kt]", "max_line_length = off"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), GOOGLE_FORMAT);
}

#[test]
fn overrides_block_indent_based_on_editorconfig_indent_size() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.kt]", "indent_size = 3"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), FormattingOptions { block_indent: 3, ..GOOGLE_FORMAT });
}

#[test]
fn doesnt_override_block_indent_when_indent_size_tab_and_no_tab_width() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.kt]", "indent_size = tab"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), GOOGLE_FORMAT);
}

#[test]
fn overrides_block_indent_with_tab_width_when_indent_size_tab() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.kt]", "indent_size = tab", "tab_width = 8"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), FormattingOptions { block_indent: 8, ..GOOGLE_FORMAT });
}

#[test]
fn overrides_block_indent_based_on_editorconfig_ij_kotlin_indent_size() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.kt]", "ij_kotlin_indent_size = 3"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), FormattingOptions { block_indent: 3, ..GOOGLE_FORMAT });
}

#[test]
fn ij_kotlin_indent_size_takes_priority_over_indent_size() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.kt]", "indent_size = 2", "ij_kotlin_indent_size = 3"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), FormattingOptions { block_indent: 3, ..GOOGLE_FORMAT });
}

#[test]
fn overrides_continuation_indent_based_on_editorconfig_ij_continuation_indent_size() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.kt]", "ij_continuation_indent_size = 3"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), FormattingOptions { continuation_indent: 3, ..GOOGLE_FORMAT });
}

#[test]
fn overrides_continuation_indent_based_on_editorconfig_ij_kotlin_continuation_indent_size() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.kt]", "ij_kotlin_continuation_indent_size = 3"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), FormattingOptions { continuation_indent: 3, ..GOOGLE_FORMAT });
}

#[test]
fn ij_kotlin_continuation_indent_size_takes_precedence_over_ij_continuation_indent_size() {
    let root = Root::new();
    root.write_conf(
        ".editorconfig",
        &["root = true", "[*.kt]", "ij_continuation_indent_size = 6", "ij_kotlin_continuation_indent_size = 3"],
    );
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), FormattingOptions { continuation_indent: 3, ..GOOGLE_FORMAT });
}

#[test]
fn overrides_trailing_comma_management_strategy_based_on_editorconfig_ktfmt_trailing_comma_management_strategy() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.kt]", "ktfmt_trailing_comma_management_strategy = only_add"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), FormattingOptions { trailing_comma_management_strategy: ONLY_ADD, ..GOOGLE_FORMAT });
}

#[test]
fn ignores_invalid_ktfmt_trailing_comma_management_strategy() {
    let root = Root::new();
    root.write_conf(".editorconfig", &["root = true", "[*.kt]", "ktfmt_trailing_comma_management_strategy = whatever"]);
    let file = root.resolve("src/main/kotlin/Example.kt");
    assert_eq!(resolve(&file), GOOGLE_FORMAT);
}

#[test]
fn combines_multiple_matching_configs() {
    let root = Root::new();
    root.write_conf(
        ".editorconfig",
        &[
            "root = true",
            "[{*.kt,*.kts}]",
            "indent_size = 3",
            "ktfmt_trailing_comma_management_strategy = none",
            "[src/**/*.kts]",
            "max_line_length = 120",
        ],
    );
    let root_options = FormattingOptions {
        block_indent: 3,
        trailing_comma_management_strategy: TrailingCommaManagementStrategy::None,
        ..GOOGLE_FORMAT
    };

    root.write_conf(
        "src/main/.editorconfig",
        &[
            "[*.kt]",
            "ij_continuation_indent_size = 3",
            "max_line_length = 200",
            "ktfmt_trailing_comma_management_strategy = only_add",
        ],
    );
    let main_options = FormattingOptions {
        max_width: 200,
        block_indent: 3, // inherited from root .editorconfig
        continuation_indent: 3,
        trailing_comma_management_strategy: ONLY_ADD, // overridden from root .editorconfig
        ..GOOGLE_FORMAT
    };

    root.write_conf(
        "src/test/.editorconfig",
        &["root = true", "[*.kt]", "indent_size = 4", "ij_continuation_indent_size = 2", "max_line_length = 300"],
    );
    // trailing comma not inherited from root as test/.editorconfig marked root=true
    let test_options = FormattingOptions { max_width: 300, block_indent: 4, continuation_indent: 2, ..GOOGLE_FORMAT };

    assert_eq!(resolve(&root.resolve("build.gradle.kts")), root_options);
    assert_eq!(resolve(&root.resolve("src/main/kotlin/Example.kt")), main_options);
    assert_eq!(resolve(&root.resolve("src/test/kotlin/ExampleTest.kt")), test_options);
    assert_eq!(
        resolve(&root.resolve("src/main/kotlin/ExampleTest.kts")),
        FormattingOptions { max_width: 120, ..root_options }
    );
    // root=true stops even non-matching fall-through
    assert_eq!(resolve(&root.resolve("src/test/kotlin/ExampleTest.kts")), GOOGLE_FORMAT);
}
