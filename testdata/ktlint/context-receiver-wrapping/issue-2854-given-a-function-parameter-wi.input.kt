fun bar1(foo: context(Foo) () -> Unit = { foobar() }) {}
fun bar2(
    foo: context(Foo) () -> Unit = { foobar() }
) {}