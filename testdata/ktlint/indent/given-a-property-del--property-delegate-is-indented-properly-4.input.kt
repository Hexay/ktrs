fun lazyList() = lazy { mutableListOf<String>() }

class Test {
    val list: List<String>
        by lazyList()

    val aVar = 0
}