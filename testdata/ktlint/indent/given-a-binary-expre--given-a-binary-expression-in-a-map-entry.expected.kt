val foo1 =
    "bar" to
        "a" +
        "b" +
        "c"
val foo2 =
    "bar" to
        "a"
            .plus("b")
            .plus("c")