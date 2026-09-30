fun foo(bar: Any): String = when(bar) {
    1,
    2,
    -> {
        "a"
    }
    3,
    4,
    -> {
        "b"
    }
    5,
    6 /* some comment */,
    -> {
        "c"
    }
    else -> {
        "d"
    }
}