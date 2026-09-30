class Bar : Foo() {
    override public val v = ""
    suspend override fun f(v: Any): Any = ""
    tailrec override fun foo(bar: String): String = foo(bar.substringBeforeLast(" "))
    override @Annotation fun getSomething() = ""
    suspend @Annotation override public @Woohoo(data = "woohoo") fun doSomething() = ""
}