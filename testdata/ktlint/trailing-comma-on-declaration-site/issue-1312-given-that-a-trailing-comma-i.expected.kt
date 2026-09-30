fun foo(bar: Int): String = when(bar) {
    1, 2 -> "a"
    3,
    4,
    -> "b"
    5,
    6,
    -> "c"
    else -> "d"
}