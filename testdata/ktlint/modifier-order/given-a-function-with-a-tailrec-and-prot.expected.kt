class A {
    protected tailrec fun foo(bar: String): String = foo(bar.substringBeforeLast("\n"))
}