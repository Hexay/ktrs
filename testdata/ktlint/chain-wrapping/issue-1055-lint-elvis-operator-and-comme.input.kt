fun test(): Int {
    val foo = foo()
        ?: // Comment
        return bar()
    return baz()
}

fun foo(): Int? = null
fun bar(): Int = 1
fun baz(): Int = 2