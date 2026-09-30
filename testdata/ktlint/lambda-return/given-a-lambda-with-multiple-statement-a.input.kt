val foo = bar {
    if (baz()) return@bar "value"

    return@bar "value"
}