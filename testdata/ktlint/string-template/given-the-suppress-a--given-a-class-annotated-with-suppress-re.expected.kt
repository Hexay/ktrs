class Foo1 {
    val foo
        get() = "Foo = $foo"
}
@Suppress("RemoveCurlyBracesFromTemplate")
class Foo2 {
    val foo
        get() = "Foo = ${foo}"
}
@Suppress("RemoveCurlyBracesFromTemplate")
class Foo3 {
    val foo
        get() = "Foo = ${foo}"
}