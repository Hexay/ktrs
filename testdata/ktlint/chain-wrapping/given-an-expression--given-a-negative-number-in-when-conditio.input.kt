fun foo() = 1
val foo = when (foo()) {
    -1 -> "a"
    0 -> "b"
    else -> "c"
}