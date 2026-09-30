// Max line length marker:                                 #
val foo =
    Bar
        .buildernnnnnnnnnnnnnnnnnnnnnnnnnnnnnn()
        .baz(1)
        .build()

val foo1 =
    requireNotNull(bar.filter { it == "bar" }) {
        "some message"
    }
val foo2 =
    requireNotNull(bar?.filter { it == "bar" }) {
        "some message"
    }