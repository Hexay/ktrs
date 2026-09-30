fun foo1(vararg a: Any) = "some-result"
inline fun foo2(noinline bar: () -> Unit) {
    bar()
}
inline fun foo3(crossinline bar: () -> Unit) {
    bar()
}