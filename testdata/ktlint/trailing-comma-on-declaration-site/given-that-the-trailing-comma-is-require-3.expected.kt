data class Foo1(val bar: Int)
data class Foo2(
    val bar: Int, // The comma should be inserted before the comment
)
data class Foo3(
    val bar: Int, /* The comma should be inserted before the comment */
)