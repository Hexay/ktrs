fun foo() {
    val strings = listOf("a")
    println(
        """
${strings.joinToString { "a" }}
$strings
        """
    )
}
val foo =
    """
${strings.joinToString { "a" }}
$strings
    """.trimIndent()