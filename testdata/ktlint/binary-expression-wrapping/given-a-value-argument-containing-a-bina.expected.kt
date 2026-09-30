// Max line length marker:    #
val foobar1 = Foo(
    1 * 2 * 3 * 4
)
val foobar2 = Foo(
    bar(1 * 2 * 3)
)
val foobar3 = Foo(
    bar(
        "bar" + "bazzzzzzzzzzz"
    )
)
val foobar4 = Foo(
    bar(
        "bar" +
            "bazzzzzzzzzzzz"
    )
)
@Suppress("ktlint:standard:max-line-length")
val foobar5 = Foo(bar("bar" + "bazzzzzzzzzzzz"))