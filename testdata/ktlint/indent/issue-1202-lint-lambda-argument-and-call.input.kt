class Foo {
    fun bar() {
        val foo = bar.associateBy({ item -> item.toString() }, ::someFunction).toMap()
    }
}