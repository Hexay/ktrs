fun foo(): Any {
    return if (true) {
        Foo()
    } else {
        return Bar()
    }
}