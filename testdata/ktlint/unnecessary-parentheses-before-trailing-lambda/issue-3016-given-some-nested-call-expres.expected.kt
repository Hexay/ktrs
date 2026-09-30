class Foo
class Bar
class Baz
operator fun Foo.invoke() = Bar()
operator fun Bar.invoke() = Baz()
inline operator fun Baz.invoke(function: () -> Unit) {}

fun main() {
    val foo = Foo()
    foo()()() {} // Do not remove empty parameter list
    val bar = Bar()
    bar()() {} // Do not remove empty parameter list
    val baz = Baz()
    baz {} // Empty parameter list is to be removed
}