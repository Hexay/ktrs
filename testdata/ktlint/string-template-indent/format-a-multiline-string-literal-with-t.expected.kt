fun foo() {
    println(
        """
Some text starting at the beginning of the line

    Some text not starting at the beginning of the line
        """.trimIndent()
    )
}