fun bar1(): Int? = null
fun bar2(): List<Int> = emptyList()
fun foo(): Int? =
    bar1().also {
        println("bar1")
    }
    ?: bar2().also {
        println("bar2")
    }