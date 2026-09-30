data class Foo(val bar: String)
val foo = Foo("foobar").also {
    println("Bar = ${it.bar}") // In real code the $ would not have been escaped
}