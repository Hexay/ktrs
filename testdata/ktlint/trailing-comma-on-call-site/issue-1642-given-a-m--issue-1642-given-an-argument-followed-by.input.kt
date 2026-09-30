fun main() {
    bar(
        "foo",
        object : Foo {
            override fun foo() {
                TODO("Not yet implemented")
            }
        }
    )
}