fun foo(i: Int) = true

fun test(i: Int) {
    do {
        println()
    } while (foo(
            i
        )
    )
}