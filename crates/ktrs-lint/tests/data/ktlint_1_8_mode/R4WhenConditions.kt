fun f(x: Int) =
    when (x) {
        1 -> "a" // trailing
        2 -> "b"
        else -> "c"
    }

fun g(x: Int) =
    when (x) {
        1 -> {
            "a"
        } // c
        2 -> "b"
        else -> "c"
    }
