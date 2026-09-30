fun foo() {
    println(
    """
    """ // Indent of this line will not be fixed
    )
    println(
    """
    """.trimIndent()
    )
    println(
    """
    """.trimMargin()
    )
}