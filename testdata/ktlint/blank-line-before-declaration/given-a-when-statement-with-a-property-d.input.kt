val foo =
    when (val foobar = FooBar()) {
        is Bar -> foobar.bar()
        is Foo -> foobar.foo()
        else -> foobar
    }