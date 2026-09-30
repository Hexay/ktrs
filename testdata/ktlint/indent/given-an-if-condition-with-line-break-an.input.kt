// https://github.com/ktlint/ktlint/issues/871
fun function(param1: Int, param2: Int, param3: Int?): Boolean {
    return if (
        listOf(
            param1,
            param2,
            param3
        ).none { it != null }
    ) {
        true
    } else {
        false
    }
}

// https://github.com/ktlint/ktlint/issues/900
enum class Letter(val value: String) {
    A("a"),
    B("b");
}
fun broken(key: String): Letter {
    for (letter in Letter.values()) {
        if (
            letter.value
                .equals(
                    key,
                    ignoreCase = true
                )
        ) {
            return letter
        }
    }
    return Letter.B
}