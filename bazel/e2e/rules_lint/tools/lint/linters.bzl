"The ktlint aspect of rules_lint, running ktrs' native `ktlint`."

load("@aspect_rules_lint//lint:ktlint.bzl", "lint_ktlint_aspect")
load("@aspect_rules_lint//lint:lint_test.bzl", "lint_test")

ktlint = lint_ktlint_aspect(
    binary = Label("@multitool//tools/ktlint"),
    editorconfig = Label("//:.editorconfig"),
    # Required: rules_lint 2.9.1 rejects None ("Aspect attribute '_baseline_file' has no default value").
    baseline_file = Label("//:ktlint-baseline.xml"),
    # ktlint logs to stdout, which is the report.
    args = ["--log-level=none"],
)

ktlint_test = lint_test(aspect = ktlint)
