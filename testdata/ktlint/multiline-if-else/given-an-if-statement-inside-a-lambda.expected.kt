fun test(s: String?): Int {
    val i = s.let {
        if (it == "") {
            1
        } else {
            2
        }
    } ?: 0
    return i
}