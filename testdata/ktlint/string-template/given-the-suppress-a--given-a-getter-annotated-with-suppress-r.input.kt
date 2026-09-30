class Foo {
    val foo1
        get() = "Foo = ${foo}"

    @Suppress("RemoveCurlyBracesFromTemplate")
    val foo2
        get() = "Foo = ${foo}"

    @Suppress("RemoveCurlyBracesFromTemplate", "OtherSuppression")
    val foo3
        get() = "Foo = ${foo}"
}