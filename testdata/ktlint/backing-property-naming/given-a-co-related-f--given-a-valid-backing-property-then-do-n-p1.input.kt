class Foo {
    fun getFoo(): String = _foo

    private companion object {
        var _foo = "some-value"
    }
}