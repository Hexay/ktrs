fun foo(bar: Int): String = when(bar) {
    1, 2 -> "a"
    3, 4, // The comma should be inserted before the comment
    -> "a"
    5,
    6, /* The comma should be inserted before the comment */
    -> "a"
    else -> "b"
}