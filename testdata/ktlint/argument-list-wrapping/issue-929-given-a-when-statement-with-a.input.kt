fun foo(i: Int) = true

fun test(i: Int) {
    when (foo(
        i
    )) {
        true -> println(1)
        false -> println(2)
    }
}