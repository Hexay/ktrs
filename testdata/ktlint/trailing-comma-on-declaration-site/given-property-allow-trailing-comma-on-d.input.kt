data class Foo1(val bar: Int,)

class Foo2<A, B,> {}

fun foo3(bar: Int): String = when(bar) {
    1, 2, -> "a"
    else -> "b"
}

fun foo4() {
    fun bar(): Pair<Int, Int> = Pair(1, 2)

    val (x, y,) = bar()
    val [a, b,] = bar()
}

val foo5: (Int, Int,) -> Int = 42

val foo6: (Int, Int,) -> Int = { foo, bar, -> foo * bar }