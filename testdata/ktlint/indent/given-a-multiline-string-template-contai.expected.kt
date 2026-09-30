fun foo1() =
    "Sum of uneven numbers = ${
        listOf(1,2,3)
            .filter { it % 2 == 0 }
            .sum()
    }"
fun foo2() = "Sum of uneven numbers = ${
    listOf(1,2,3)
        .filter { it % 2 == 0 }
        .sum()
}"