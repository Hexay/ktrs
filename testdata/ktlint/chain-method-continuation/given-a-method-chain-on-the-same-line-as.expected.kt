val foo = listOf(1, 2, 3, 4)
    .filter { it > 2 }
    .filter { it > 3 }
    .filter {
        it > 4
    }.sum()
    .dec()