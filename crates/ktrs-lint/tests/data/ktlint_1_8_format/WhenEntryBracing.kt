fun foo(a: Any) =
    when (a) {
        is Int, is Long -> 1
        is String,
        is Char -> 2 // two
        else -> {
            3
        }
    }
