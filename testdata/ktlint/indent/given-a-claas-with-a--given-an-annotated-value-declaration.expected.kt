class Foo {
    @Deprecated("Foo") val foo1 = "foo"

    @Deprecated("Foo")
    val foo2 = "foo"
}

@Deprecated("Foo")
val foo3 = "foo"