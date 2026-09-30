class Foo {
    val foo: String
        get() = _foo

    companion object {
        private var _foo = "some-value"
    }
}