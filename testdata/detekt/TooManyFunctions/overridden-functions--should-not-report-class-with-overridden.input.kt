interface I1 {
    fun func1()
    fun func2()
}

class Foo : I1 {
    override fun func1() = Unit
    override fun func2() = Unit
}