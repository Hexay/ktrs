fun foo() =
    @Foo1 @Foo2
    @Foo3("bar3")
    @Foo4
    baz()
fun bar() =
    @Bar1 @Bar2 @Bar3("bar3") @Bar4
    baz()