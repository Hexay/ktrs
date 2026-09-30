fun foo() {
    shouldFail<IllegalArgumentException>(sinceKotlin = "255.255.255") {
        // no-op
    }
}