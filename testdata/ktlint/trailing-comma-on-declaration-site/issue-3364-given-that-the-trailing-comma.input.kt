fun foo() {
    fun bar(): Pair<Int, Int> = Pair(1, 2)

    val [x, y] = bar()
    val [
        x,
        y // The comma should be inserted before the comment
    ] = bar()
    val [
        x,
        y /* The comma should be inserted before the comment */
    ] = bar()
}