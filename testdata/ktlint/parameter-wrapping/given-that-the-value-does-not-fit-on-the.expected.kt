// Max line length marker:                   #
class Bar(
    val foooooooooooooooooTooLong: Foo =
        Foo(),
    val foooooooooooooNotTooLong: Foo = Foo(),
)
fun bar(
    foooooooooooooooooooooTooLong: Foo =
        Foo(),
    foooooooooooooooooNotTooLong: Foo = Foo(),
)