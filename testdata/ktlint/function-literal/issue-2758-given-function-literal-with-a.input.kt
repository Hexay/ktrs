val foo = if (cond) {
    { -> "bar" }
} else {
    { -> "baz" }
}