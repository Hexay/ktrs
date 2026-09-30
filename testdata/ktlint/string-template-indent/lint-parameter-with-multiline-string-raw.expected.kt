val result =
    when {
        someBooleanFunction() ->
            """
            someText
            """.trimIndent()
        else ->
            """
            someOtherText
            """.trimIndent()
    }