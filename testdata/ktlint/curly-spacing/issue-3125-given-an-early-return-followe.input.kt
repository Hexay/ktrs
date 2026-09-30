fun foo(bar: String?) {
    bar ?: return
    { print(bar) }
}