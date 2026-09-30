// Max line length marker:  #
context(
    _: Foooooooooooooooo<
        Foo,
        Bar
        >
)
fun fooBar1()

@Suppress("ktlint:standard:max-line-length")
context(_: Foooooooooooooooo<Foo, Bar>)
fun fooBar2()