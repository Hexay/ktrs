val foo1 = listOf(1, 2, 3) /* 1 */
    .filter { it > 2 }!! /* 2 */
    .takeIf { it.count() > 100 } /* 3 */
    ?.sum()!!
val foo2 = listOf(1, 2, 3) // 1
    .filter { it > 2 }!! // 2
    .takeIf { it.count() > 100 } // 3
    ?.sum()!!