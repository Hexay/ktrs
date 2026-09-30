fun foo() {
    println("Foo = $foo")
    @Suppress("RemoveCurlyBracesFromTemplate")
    println("Foo = ${foo}")
    @Suppress("RemoveCurlyBracesFromTemplate", "OtherSuppression")
    println("Foo = ${foo}")
}