private var foo: String =
    "foo"
    set(value) {
        listOf("a", value, "c")
            .filterNotNull()
            .joinToString()
    }