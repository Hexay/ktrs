private val foo1: String =
    "foo"
    get() {
        return listOf("a", value, "c")
            .filterNotNull()
            .joinToString()
    }