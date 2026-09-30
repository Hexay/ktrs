class Foo
fun bar(foo: KFunction0<Foo>) = println(foo())
fun main() {
    bar(::Foo)
    bar(::Foo)
    bar(
        ::Foo
    )
}