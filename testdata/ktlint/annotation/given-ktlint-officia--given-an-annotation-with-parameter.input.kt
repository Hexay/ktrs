data class Foo @Bar1 @Bar2("bar") @Bar3 @Bar4 constructor(private val foobar: Int) {
    fun foo(): String = "foo"
}
data class Baz @Baz1 @Baz2("bar") @Baz3 @Baz4 constructor(private val foobar: Int) {
    fun foo(): String = "foo"
}