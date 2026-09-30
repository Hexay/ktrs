class Foo {
    val foo: String
        get() = _foo

    private companion object {
        var _foo = "some-value"
    }
}