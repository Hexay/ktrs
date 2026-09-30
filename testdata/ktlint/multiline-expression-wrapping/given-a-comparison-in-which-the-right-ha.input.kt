fun foo(bar: String): Boolean {
    return bar != """
        some text
    """.trimIndent()
}