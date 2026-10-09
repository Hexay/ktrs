fun test(x: Int): Int {
    val y = x ?: return 0
    when (x) {
        5 -> println("x=5")
        4 -> return 4
    }
    return 6
}