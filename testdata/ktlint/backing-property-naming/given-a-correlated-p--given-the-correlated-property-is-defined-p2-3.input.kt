class Foo {
    val føø: String
        get() = _føø

    companion object {
        private var _føø = "some-value"
    }
}