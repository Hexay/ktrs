fun foo(bar: Int): String = when(bar) {
    1, 2 -> "a"
    3, 4 // The comma before the comment should be removed without removing the comment itself
    -> "a"
    5,
    6 /* The comma before the comment should be removed without removing the comment itself */
    -> "a"
    else -> "b"
}