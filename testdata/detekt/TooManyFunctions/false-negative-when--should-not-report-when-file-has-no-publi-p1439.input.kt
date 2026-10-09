class A {
    private fun a() = Unit
    private fun b() = Unit
    @Deprecated("")
    private fun c() = Unit
}

interface I {
    fun a() = Unit
    fun b() = Unit
}

class B : I {
    override fun a() = Unit
    override fun b() = Unit
}