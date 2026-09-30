enum class E1(bar: String) {
    ONE("one"),
    TWO("two");
    fun fn() {}
}
enum class E2(bar: String) {
    ONE("one"),
    TWO("two");
    val foo = "foo"
}
enum class E3(bar: String) {
    ONE("one"),
    TWO("two");
    private companion object {
       val foo = "foo"
   }
}