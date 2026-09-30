fun test() {
    while (System.currentTimeMillis() % 2 == 0L) {
        println(System.currentTimeMillis())
    }
    while (Random(System.currentTimeMillis()).nextBoolean()) {
        println(System.currentTimeMillis())
    }
}