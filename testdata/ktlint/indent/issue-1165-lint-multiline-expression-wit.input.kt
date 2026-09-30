fun test() {
    val a: String = ""

    val someTest: Int?

    someTest =
        a
            .toIntOrNull()
            ?: 1
}