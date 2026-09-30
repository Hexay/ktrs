open class Foo {
    open val foo = "foo"
}

val BAR = object : Foo() {
    override val foo = "bar"
}