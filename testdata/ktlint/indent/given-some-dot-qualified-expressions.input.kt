val foo1 =
nullableList
.find { !it.empty() }
?.map { x + 2 }
?.filter { true }
val foo2 =
listOf(listOf(1, 2, 3))
.map {
it
.map { it + 1 }
.filter { it > 3 }
}
.reduce { acc, curr -> acc + curr }
.toString()
val foo3 = 1