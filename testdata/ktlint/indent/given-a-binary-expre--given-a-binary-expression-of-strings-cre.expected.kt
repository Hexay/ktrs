val foo =
    listOf("foo", "bar").joinToString {
        it.toUpperCaseAsciiOnly()
    } + bar(
        "foo"
    ) + bar(
        "foo"
    )