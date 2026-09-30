fun test() {
    while (System.currentTimeMillis() % 2 == 0L) {
        println(System.currentTimeMillis())
    }
    do {
        println(System.currentTimeMillis())
    } while (System.currentTimeMillis() % 2 == 0L)
    do {
        println(System.currentTimeMillis())
    } while (System.currentTimeMillis() % 2 == 0L)
}