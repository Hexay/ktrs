fun bar1(foo: context(_: Foo) () -> Unit = { foobar() }) {}
fun bar2(
    foo: context(_: Foo) () -> Unit = { foobar() }
) {}