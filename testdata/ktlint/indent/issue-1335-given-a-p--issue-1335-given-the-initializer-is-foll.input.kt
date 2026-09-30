private val foo: String =
    "foo"
    get() =
        listOf("a", value, "c")
            .filterNotNull()
            .joinToString()