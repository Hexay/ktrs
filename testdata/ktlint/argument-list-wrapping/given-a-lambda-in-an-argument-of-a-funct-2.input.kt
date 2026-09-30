val foo1 =
    foo(
        a = "1", b = {
        it.toString()
    })
val foo2 =
    foo("1",
        { it.toString() })