val foo = when (1) {
    1 -> if (true) {
        2
    } else {
        3
    }
    2 -> 1.let {
        it + 1
    }.let {
        it + 1
    }
    else -> 0
}