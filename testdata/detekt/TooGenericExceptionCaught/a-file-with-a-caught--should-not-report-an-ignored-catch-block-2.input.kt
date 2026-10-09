fun f() {
    try {
        throw Throwable()
    } catch (myIgnore: NullPointerException) {
        throw Error()
    }
}