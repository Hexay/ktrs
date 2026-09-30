// Max line length marker:     #
context(_: Fooooooooooooooooooo1, _: Foooooooooooooooooooooooooooooo2)
fun fooBar1()

@Suppress("ktlint:standard:max-line-length")
context(_: Fooooooooooooooooooo1, _: Foooooooooooooooooooooooooooooo2)
fun fooBar2()

class Bar {
    context(_: Fooooooooooooooo1, _: Foooooooooooooooooooooooooo2)
    fun fooBar3()
}