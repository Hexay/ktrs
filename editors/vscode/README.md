# ktrs for VS Code

ktlint diagnostics, quick fixes and ktfmt or ktlint formatting for Kotlin, from [ktrs](https://github.com/Hexay/ktrs)'s
native language server (`ktrs lsp`). No JVM, a few milliseconds per file.

- **Diagnostics**: ktlint violations as you type, with the rule id as the code.
- **Quick fixes**: fix one violation, fix all autocorrectable ones, suppress a rule on the line or in the file.
- **Formatting**: `Format Document` with ktfmt or ktlint's `--format`.
- **Build-aware**: reads the setup from Gradle or Maven (ktfmt-gradle, ktlint-gradle, kotlinter, Spotless, the Maven
  plugins, convention plugins, the version catalog). Without one it shows ktlint 1.8 diagnostics and doesn't format.

It contributes the `kotlin` language for `.kt` and `.kts` but no syntax highlighting or code intelligence: install a
Kotlin extension for those (for example JetBrains' Kotlin); ktrs runs next to it.

## Setup

Format on save with ktrs, and fix all ktlint violations on save:

```json
"[kotlin]": {
  "editor.defaultFormatter": "hexay.ktrs",
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": { "source.fixAll.ktlint": "explicit" }
}
```

Formatting is off unless the build configures ktfmt or ktlint, or you pick one:

```json
"ktrs.format.tool": "ktfmt",
"ktrs.ktfmt.style": "kotlinlang"
```

## Settings

Every `ktrs.*` setting except `ktrs.path` and `ktrs.trace.server` is a key of the server's settings, listed with
their values and meaning in [`crates/ktrs-lsp/src/lib.rs`](https://github.com/Hexay/ktrs/blob/master/crates/ktrs-lsp/src/lib.rs)
(`ktrs.ktfmt.style` is `ktfmt.style` there). They override the build's values key by key; `null`, the default, leaves
the build's value or, without one, the server's default.

The server logs the effective configuration and where it came from to the ktrs output (**ktrs: Show Output**).
**ktrs: Restart Server** restarts it.

## The binary

The extension for Windows, macOS, Linux and Alpine (x64 and arm64) bundles `ktrs`. Set `ktrs.path` to run another
one, such as a newer release or your own build. On other platforms (the universal package) it runs `ktrs` from `PATH`:
[install ktrs](https://github.com/Hexay/ktrs#installation). In an untrusted workspace, `ktrs.path` from the
workspace's settings is ignored.
