class Bar : Foo() {
    public override val v = ""
    override suspend fun f(v: Any): Any = ""
    override tailrec fun foo(bar: String): String = foo(bar.substringBeforeLast(" "))
    @Annotation override fun getSomething() = ""
    @Annotation @Woohoo(data = "woohoo") public override suspend fun doSomething() = ""
}