fun test(x: Int): Int {
    if (x < 4) {
        println("x x is less than 4")
        if (x < 2) {
          println("x is also less than 2")
          return 1
        }
        return 0
    }
    when (x) {
        5 -> println("x=5")
        4 -> return 4
    }
    return 6
}