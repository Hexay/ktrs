class Foo {
    private val foo
        by option("--myOption")
            .int()
            .default(1)
}