fun foo() {
    when {
        // comment
        true -> {
        }
    }
    when {
        1, // first element
        2 // second element
        -> true
    }
}