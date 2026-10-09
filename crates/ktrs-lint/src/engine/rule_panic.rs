//! Rule exceptions are panics that the engine catches and reports as `KtLintRuleException`, as ktlint does with
//! Java exceptions; see `ktrs_syntax::caught_panic`. Their message starts with the Kotlin exception's simple
//! name (crate docs); [`jvm_throwable_string`] gives the `Throwable.toString()` the JVM prints for it.

pub(crate) use ktrs_syntax::caught_panic::catch_quietly as catch_rule_panic;
pub use ktrs_syntax::caught_panic::silence_caught_panics as silence_caught_rule_panics;

/// `(name, message)` of `"Name: message"` / `"Name"`.
fn exception_name(cause: &str) -> (&str, Option<&str>) {
    match cause.split_once(": ") {
        Some((name, message)) => (name, Some(message)),
        None => (cause, None),
    }
}

/// The class the JVM throws for the Kotlin construct our panics name (`require` -> IllegalArgumentException,
/// `check`/`error` -> IllegalStateException, `first()` -> NoSuchElementException, `lateinit` ->
/// UninitializedPropertyAccessException, `TODO()` -> NotImplementedError).
fn jvm_class_name(name: &str) -> Option<&'static str> {
    Some(match name {
        "ArithmeticException" => "java.lang.ArithmeticException",
        "AssertionError" => "java.lang.AssertionError",
        "ClassCastException" => "java.lang.ClassCastException",
        "IllegalArgumentException" => "java.lang.IllegalArgumentException",
        "IllegalStateException" => "java.lang.IllegalStateException",
        "IndexOutOfBoundsException" => "java.lang.IndexOutOfBoundsException",
        "NullPointerException" => "java.lang.NullPointerException",
        "StringIndexOutOfBoundsException" => "java.lang.StringIndexOutOfBoundsException",
        "UnsupportedOperationException" => "java.lang.UnsupportedOperationException",
        "NoSuchElementException" => "java.util.NoSuchElementException",
        "PatternSyntaxException" => "java.util.regex.PatternSyntaxException",
        "NotImplementedError" => "kotlin.NotImplementedError",
        "UninitializedPropertyAccessException" => "kotlin.UninitializedPropertyAccessException",
        "DeprecatedEditorConfigPropertyException" => {
            "io.github.ktlint.core.rule.engine.core.api.editorconfig.DeprecatedEditorConfigPropertyException"
        }
        _ => return None,
    })
}

/// A rule exception's cause as the JVM prints it: the leading Kotlin exception name made fully qualified
/// (ktlint's own classes in 2.0's package). A message that names no known exception is returned as is.
pub fn jvm_throwable_string(cause: &str) -> String {
    let (name, message) = exception_name(cause);
    let Some(class) = jvm_class_name(name) else {
        return cause.to_owned();
    };
    match message {
        // `!!` throws a NullPointerException without message; ours names the expression.
        Some(m) if class == "java.lang.NullPointerException" && m.ends_with("!!") => class.to_owned(),
        Some(m) => format!("{class}: {m}"),
        None => class.to_owned(),
    }
}
