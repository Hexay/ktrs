fun foo(i: Int) = true

fun test(i: Int) {
    while (foo(
            i
        )
    ) {
        println()
    }
}