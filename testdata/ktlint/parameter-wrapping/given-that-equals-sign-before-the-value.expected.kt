// Max line length marker:            #
class Bar1(
    val foooooooooooooooooTooLong: Foo =
        Foo(),
    val foooooooooooooNotTooLong: Foo =
        Foo(),
)

@Suppress("ktlint:standard:max-line-length")
class Bar2(
    val foooooooooooooooooTooLong: Foo = Foo(),
    val foooooooooooooNotTooLong: Foo = Foo(),
)

fun bar1(
    foooooooooooooooooooooTooLong: Foo =
        Foo(),
    foooooooooooooooooNotTooLong: Foo =
        Foo(),
)

@Suppress("ktlint:standard:max-line-length")
fun bar2(
    foooooooooooooooooooooTooLong: Foo = Foo(),
    foooooooooooooooooNotTooLong: Foo = Foo(),
)