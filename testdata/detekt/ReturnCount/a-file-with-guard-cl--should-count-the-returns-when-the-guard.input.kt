fun test(a: Int?, b: Int?): Int {
    val first = a.toString()
    if (a == null) return 0
    val second = b.toString()
    if (b == null) return 1
    return first.length + second.length
}