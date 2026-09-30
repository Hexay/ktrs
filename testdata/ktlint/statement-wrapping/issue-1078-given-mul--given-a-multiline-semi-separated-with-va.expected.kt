fun test() {
    val a = 0
    val b = 0
    fun bar() {
        // no-op
    }
    for(i in 0..10) {
        println(i)
        println(i)
        a++
        println(a)
    }
}