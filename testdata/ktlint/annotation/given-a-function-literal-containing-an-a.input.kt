val foo1 = {
    @Bar("bar")
    foobar { "foobar" }
}
val foo2 = { @Bar("bar") foobar { "foobar" } }
val foo3 = { @Baz("baz") foobar { "foobar" } }