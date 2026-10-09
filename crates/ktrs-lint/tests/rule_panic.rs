//! The JVM `toString()` of a rule exception's cause. The quiet catch itself: ktrs-syntax `tests/caught_panic.rs`.

use ktrs_lint::engine::jvm_throwable_string;

#[test]
fn jvm_throwable_string_qualifies_the_kotlin_exception_name() {
    assert_eq!(
        jvm_throwable_string("IllegalArgumentException: Stack should be empty:\n\tx"),
        "java.lang.IllegalArgumentException: Stack should be empty:\n\tx"
    );
    assert_eq!(jvm_throwable_string("NoSuchElementException: List is empty."), "java.util.NoSuchElementException: List is empty.");
    assert_eq!(
        jvm_throwable_string("UninitializedPropertyAccessException: lateinit property x has not been initialized"),
        "kotlin.UninitializedPropertyAccessException: lateinit property x has not been initialized"
    );
    assert_eq!(jvm_throwable_string("IndexOutOfBoundsException"), "java.lang.IndexOutOfBoundsException");
    assert_eq!(jvm_throwable_string("NullPointerException: parent!!"), "java.lang.NullPointerException");
    assert_eq!(jvm_throwable_string("NullPointerException: firstChildNode"), "java.lang.NullPointerException: firstChildNode");
    assert_eq!(jvm_throwable_string("called `Option::unwrap()` on a `None` value"), "called `Option::unwrap()` on a `None` value");
}
