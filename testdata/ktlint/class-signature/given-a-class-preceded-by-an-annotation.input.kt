// Max line length marker:              #
internal class Foo1(foo1: Foo, foo2: Foo)

@Bar
internal class Foo2(foo1: Foo, foo2: Foo)

@[Bar]
internal class Foo2(foo1: Foo, foo2: Foo)

@[Bar1 Bar2 Bar3 Bar4 Bar5 Bar6 Bar7 Bar8 Bar9]
internal class Foo2(foo1: Foo, foo2: Foo)

@[Bar1 // some comment
Bar2]
internal class Foo2(foo1: Foo, foo2: Foo)