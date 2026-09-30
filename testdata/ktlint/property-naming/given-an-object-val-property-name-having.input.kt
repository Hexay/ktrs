class Foo {
    companion object {
        val foo
            get() = foobar() // Lint can not check whether data is immutable
    }
}