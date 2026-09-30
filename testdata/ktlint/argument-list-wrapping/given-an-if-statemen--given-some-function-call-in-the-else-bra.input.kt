fun foo(i: Int, j: Int) = 1

fun test() {
    val x = if (true) 1 else foo(
        2,
        3
    )
}