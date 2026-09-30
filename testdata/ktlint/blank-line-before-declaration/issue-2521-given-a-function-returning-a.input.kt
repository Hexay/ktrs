fun foo(bar: Int): (Int) -> Int {
    return fun(baz: Int): Int {
        return bar + baz
    }
}