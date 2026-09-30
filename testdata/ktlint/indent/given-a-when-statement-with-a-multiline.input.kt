fun foo() {
    return when (
        bar()
            .filter { it > 0 }
            .min()
    ) {
        1 -> "A"
        2 -> "B"
        else -> "C"
    }
}