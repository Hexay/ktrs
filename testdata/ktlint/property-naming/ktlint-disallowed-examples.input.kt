const val fooBar = "FOO-BAR" // By definition deeply immutable

var FOO2: Foo = Foo() // By definition not immutable

class Bar {
    val FOO_BAR = "FOO-BAR" // Class properties always start with lowercase, const is not allowed
}