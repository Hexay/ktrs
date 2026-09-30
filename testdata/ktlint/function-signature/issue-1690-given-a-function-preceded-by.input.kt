// Max line length marker:                   #
internal fun foo1(foo1: Foo, foo2: Foo): Foo =
    "foooooooooooooooooooooooooooooooooooooo"

@Bar
internal fun foo2(foo1: Foo, foo2: Foo): Foo =
    "foooooooooooooooooooooooooooooooooooooo"

@[Bar]
internal fun foo2(foo1: Foo, foo2: Foo): Foo =
    "foooooooooooooooooooooooooooooooooooooo"

@[Bar1 Bar2 Bar3 Bar4 Bar5 Bar6 Bar7 Bar8 Bar9]
internal fun foo2(foo1: Foo, foo2: Foo): Foo =
    "foooooooooooooooooooooooooooooooooooooo"

@[Bar1 // some comment
Bar2]
internal fun foo2(foo1: Foo, foo2: Foo): Foo =
    "foooooooooooooooooooooooooooooooooooooo"