val x1 =
    when (true) {
        true if foo("a") -> true
        else -> false
    }
val x2 =
    when (true) {
        true if foo(
            "a",
        )
        -> true

        else -> false
    }