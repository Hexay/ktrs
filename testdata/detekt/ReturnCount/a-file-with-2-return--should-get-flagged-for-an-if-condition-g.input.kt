fun test(x: Int): Int {
    when (x) {
        5 -> println("x=5")
        4 -> return 4
    }
    if (x < 4) return 0
    return 6
}