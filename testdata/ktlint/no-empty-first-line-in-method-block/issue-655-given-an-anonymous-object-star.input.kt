fun fooBuilder() = object : Foo {

    override fun foo() {
        bar()
    }
}