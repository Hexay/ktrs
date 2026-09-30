fun fooBar1() {
    // Removing this semicolon on next line results in compile errors!
    foo("some-foo");
    {
        // Do something
    }.bar()
}

fun fooBar2() {
    // Removing this semicolon on next line results in compile errors!
    foo("some-foo"); { /* Do something */ }.bar()
}

// This code is here just to demonstrate why the semicolon after the call to foo is important when the
// optional lambda is omitted
fun foo(input: String, block: ((String) -> Unit)? = null) {
    if (block != null) {
        block(input)
    } else {
        input
    }
}

// This code is here just to demonstrate why the semicolon after the call to foo is important when the
// optional lambda is omitted
fun <R> (() -> R).bar() {
    // Do something
}