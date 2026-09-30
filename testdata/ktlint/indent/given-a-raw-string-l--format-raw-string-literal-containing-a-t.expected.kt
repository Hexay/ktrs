fun foo() {
    println(
        """
        ${true}

            ${true}
        """.trimIndent()
    )
    println(
"""
${true}

    ${true}
""".trimIndent()
    )
}