fun test(x: Int): Int {
    when (x) {
        5 -> println("x=5")
        4 -> return 4
        3 -> return 3
    }
    return 6
}