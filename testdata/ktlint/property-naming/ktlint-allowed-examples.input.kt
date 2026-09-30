val foo1 = Foo() // In case developer want to communicate that Foo is mutable
val FOO1 = Foo() // In case developer want to communicate that Foo is deeply immutable

const val FOO_BAR = "FOO-BAR" // By definition deeply immutable

var foo2: Foo = Foo() // By definition not immutable

class Bar {
    val foo1 = "foo1" // Class properties always start with lowercase, const is not allowed

    const val FOO_BAR = "FOO-BAR" // By definition deeply immutable

    var foo2: Foo = Foo() // By definition not immutable

    companion object {
        val foo1 = Foo() // In case developer want to communicate that Foo is mutable
        val FOO1 = Foo() // In case developer want to communicate that Foo is deeply immutable
    }
}