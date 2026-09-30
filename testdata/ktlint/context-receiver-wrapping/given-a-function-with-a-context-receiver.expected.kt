// Max line length marker:  #
context(
    Foooooooooooooooooooo
)
fun fooBar1()

@Suppress("ktlint:standard:max-line-length")
context(Foooooooooooooooooooo)
fun fooBar2()

class Bar {
    context(
        Foooooooooooooooo
    )
    fun fooBar3()
}