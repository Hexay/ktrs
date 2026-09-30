// Max line length marker:      #
class Bar1(
    val foooooooooooooooooTooLong:
        Foo,
)

@Suppress("ktlint:standard:max-line-length")
class Bar2(
    val foooooooooooooooooTooLong: Foo
)

fun bar1(
    foooooooooooooooooooooTooLong:
        Foo,
)

@Suppress("ktlint:standard:max-line-length")
fun bar2(
    foooooooooooooooooooooTooLong: Foo
)