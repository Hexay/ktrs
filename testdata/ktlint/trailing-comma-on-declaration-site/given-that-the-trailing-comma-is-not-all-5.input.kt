data class Foo1(val bar: Int,)
data class Foo2(
    val bar: Int, // The comma before the comment should be removed without removing the comment itself
)
data class Foo3(
    val bar: Int, /* The comma before the comment should be removed without removing the comment itself */
)