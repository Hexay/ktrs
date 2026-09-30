fun foo() {
    fun bar(): Pair<Int, Int> = Pair(1, 2)

    val (x, y,) = bar()
    val (
        x,
        y, // The comma before the comment should be removed without removing the comment itself
    ) = bar()
    val (
        x,
        y, /* The comma before the comment should be removed without removing the comment itself */
    ) = bar()
}