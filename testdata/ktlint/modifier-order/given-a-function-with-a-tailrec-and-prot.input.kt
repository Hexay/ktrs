class A {
    tailrec protected fun foo(bar: String): String = foo(bar.substringBeforeLast("\n"))
}