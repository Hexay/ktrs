class Bar {
    public fun foo1() = 0
    fun foo2() = 0
    fun foo3(lambda: () -> Unit) = 0

    init {
        foo1()
        foo3 {  }
        foo2()
    }
}