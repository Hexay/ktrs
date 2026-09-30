fun test(i: Int, f: (Int) -> Unit) {
    f(i)
}

fun main() {
    test(1, fun(it: Int) {
        println(it)
    })
}