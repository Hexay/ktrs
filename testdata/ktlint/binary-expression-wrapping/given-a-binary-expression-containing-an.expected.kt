// Max line length marker:        #
val foo1 =
    foo()
        ?: "foooooooooooooooooo" +
        "bar"
// Do not remove blank line below, it is relevant as both the newline of the blank line and the indent before property foo2 have to be accounted for

val foo2 =
    foo()
        ?: "foooooooooooooooooo" +
        "bar"
@Suppress("ktlint:standard:max-line-length")
val foo3 = foo() ?: "foooooooooooooooooo" +
        "bar"