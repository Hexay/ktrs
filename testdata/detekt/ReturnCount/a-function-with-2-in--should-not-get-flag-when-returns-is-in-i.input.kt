fun test(x: Int): Int {
    val a = object {
        fun test2(x: Int): Int {
            val b = object {
                fun test3(x: Int): Int {
                    when (x) {
                        5 -> println("x=5")
                        else -> return 0
                    }
                    return 6
                }
            }
            when (x) {
                5 -> println("x=5")
                else -> return 0
            }
            return 6
        }
    }
    when (x) {
        5 -> println("x=5")
        else -> return 0
    }
    return 6
}