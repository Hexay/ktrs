fun test(s: String?): Int {
    val i = s.let {
        for (i in 1..10)
            1
        while (true)
            2
        do
            3
        while (true)
    } ?: 0
    return i
}