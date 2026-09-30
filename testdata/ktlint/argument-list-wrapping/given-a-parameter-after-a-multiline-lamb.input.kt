fun test(a: Any, b: (Any) -> Any, c: Any) {
    test(a = "1", b = {
        it.toString()
    },
    c = 123)
}