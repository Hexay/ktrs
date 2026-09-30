fun foo() {
    json(
        """
        {
            "array": [
                ${function(arg1, arg2, arg3)}
            ]
        }
        """.trimIndent()
    )
}