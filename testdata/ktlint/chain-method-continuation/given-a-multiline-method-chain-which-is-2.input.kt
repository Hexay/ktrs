val foo1 = listOf(1, 2, 3)
    .filter { it > 2 }
    .sum()
val foo2 = listOf(1, 2, 3)
    .filter { it > 2 }!!
    .sum()
val foo3 = listOf(1, 2, 3)
.filter { it > 2 }!!
    ?.sum()