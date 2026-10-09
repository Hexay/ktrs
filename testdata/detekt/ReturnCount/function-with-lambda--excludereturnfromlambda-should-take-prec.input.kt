fun test() {
    listOf(1, 2, 3, 4, 5).forEach lit@{
        if (it == 3) return@lit
        if (it == 4) return@lit
    }
    return
}