fun fooBar(foo: () -> String): (() -> String) -> String = { bar -> foo().plus("  ").plus(bar()) }

val foobar = fooBar { "Hello" }() { "world" }