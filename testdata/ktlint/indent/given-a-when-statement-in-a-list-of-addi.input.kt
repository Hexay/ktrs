val foo1 = 0 + 1 + when {
    else -> 2 + 3
} + 4
val foo2 = when {
    true -> 0 + 1 + when {
        else -> 2 + 3
    } + 4
    else -> -1
}