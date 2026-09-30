val bar = when (foo) {
    1 -> true
    2 ->
        false
    3 ->
        false ||
        true
    4 -> false || foobar({
    }) // Special case which is allowed
    else -> {
        true
    }
}