class FooBar {
    @Foo("foo")
    @Bar
    val bar: Any
    @Baz("baz") @Bar
    val baz: Any
}