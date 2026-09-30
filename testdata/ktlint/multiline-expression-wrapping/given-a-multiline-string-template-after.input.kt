fun foo(bar: String) =
    when (bar) {
        "bar bar bar bar bar bar bar bar bar" -> """
            The quick brown fox
            jumps over the lazy dog
            """.trimIndent()
        else -> ""
    }