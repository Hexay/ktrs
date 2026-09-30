class Foo {
    val føø: String
        get() = _føø

    private companion object {
        var _føø = "some-value"
    }
}