fun foo(x: Int, y: Int, z: Int) {}
fun test(a: Int, b: Int, c: Int, d: Int, bar: Boolean) {
    foo(
        a,
        if (bar) {
            b
        } else {
            c
        },
        d
    )
}