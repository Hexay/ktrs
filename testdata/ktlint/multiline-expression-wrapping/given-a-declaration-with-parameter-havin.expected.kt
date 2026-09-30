fun foo(
    val string: String =
        barFoo
            .count { it == "bar" },
    val int: Int
)