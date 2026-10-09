fun test(x: Int): Int {
    if (x < 4) {
        println("x x is less than 4")
        return 0
    }
    when (x) {
        5 -> println("x=5")
        4 -> return 4
    }
    return 6
}