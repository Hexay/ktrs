fun main() {
    bar(
        object : Foo {
            override fun foo() {
                "foo"
            }
        },
    )
}