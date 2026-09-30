fun fooBar(foo: String?, bar: String) =
    foo
        ?.lowercase()
        ?: bar
            .uppercase()
            .trimIndent()