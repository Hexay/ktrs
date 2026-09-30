val foo1 = listOf("a", "b")

val foo2 = Pair(1, 2)

val foo3: List<String> = emptyList()

val foo4 = Array(2) { 42 }
val bar4 = foo4[1]

annotation class Foo5(val params: IntArray)
@Foo5([1, 2])
val foo5: Int = 0