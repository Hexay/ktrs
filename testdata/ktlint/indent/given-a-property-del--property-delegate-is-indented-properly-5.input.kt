fun lazyList(a: Int, b: Int) = lazy { mutableListOf<String>() }

class Test {
    val list: List<String>
        by lazyList(
            1,
            2
        )

    val aVar = 0
}