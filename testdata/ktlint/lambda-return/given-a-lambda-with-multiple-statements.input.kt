val foo = bar {
    if (baz()) return@bar "value"

    "value"
}