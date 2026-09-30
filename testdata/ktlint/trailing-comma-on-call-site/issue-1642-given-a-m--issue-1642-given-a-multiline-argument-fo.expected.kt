fun main() {
    bar(
        object : Foo {
            override fun foo() {
                TODO("Not yet implemented")
            }
        },
        "foo",
    )
}