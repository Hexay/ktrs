val foo1 = listOf(1, 2, 3)
    ?./** some comment */size
val foo2 = listOf(1, 2, 3)
    ?./** some comment */single()
val foo3 = listOf(1, 2, 3)
    ?./** some comment */filter { it > 2 }
val foo4 = listOf(1, 2, 3)
    ?./* some comment */size
val foo5 = listOf(1, 2, 3)
    ?./* some comment */single()
val foo6 = listOf(1, 2, 3)
    ?./* some comment */filter { it > 2 }
val foo7 = listOf(1, 2, 3)
    ?.// some comment
    size
val foo8 = listOf(1, 2, 3)
    ?.// some comment
    single()
val foo9 = listOf(1, 2, 3)
    ?.// some comment
    filter { it > 2 }