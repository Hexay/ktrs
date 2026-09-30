val foo = listOf(1, 2, 3)
    .filter {
        listOf(1, 2, 3).map { it * it }.size > 1
    }!!
    .map {
        it * it
    }