// Max line length marker:    #
val foobar = {
    it.foo().foobar().foobar2()
}
val foo =
    bar
        .filter { it > 2 }!!
        .takeIf {
            it.count() > 100
        }.map { it * it }
        ?.sum()!!