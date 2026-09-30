fun test(f: () -> Unit, g: () -> Unit) {
    f()
    g()
}

fun main() {
    test({
        println(1)
    }, {
        println(2)
    })
}