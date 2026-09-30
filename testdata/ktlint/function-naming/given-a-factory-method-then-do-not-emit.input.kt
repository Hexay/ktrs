interface Foo
class FooImpl : Foo
fun Foo(): Foo = FooImpl()
fun Bar.Foo(): Foo = FooImpl()