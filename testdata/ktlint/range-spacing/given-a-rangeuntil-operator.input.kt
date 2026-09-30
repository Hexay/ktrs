@OptIn(ExperimentalStdlibApi::class)
fun foo(int: Int): String =
    when (int) {
        in 10..<20 -> "A"
        in 20..< 30 -> "B"
        in 30 ..< 40 -> "C"
        in 40 ..<50 -> "D"
        else -> "unknown"
    }