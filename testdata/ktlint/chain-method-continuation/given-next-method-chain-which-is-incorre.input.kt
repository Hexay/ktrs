val foo = listOf(1, 2, 3)!!.
    filter { it > 2 }!!.
    filter {
        it > 2
    }!!.
    sum()