fun fooBar(foobar: String?, bar: String) =
    foo
        ?.lowercase()
        ?: bar
            .uppercase()
            .trimIndent()