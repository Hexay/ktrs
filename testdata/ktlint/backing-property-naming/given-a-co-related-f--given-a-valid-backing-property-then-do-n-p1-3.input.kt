class Foo {
    fun getFoo(): String = _foo

    companion object {
        private var _foo = "some-value"
    }
}