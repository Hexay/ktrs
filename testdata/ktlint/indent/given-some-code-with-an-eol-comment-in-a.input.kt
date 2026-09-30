fun foo(param: Foo, other: String) {
    foo(
        param = param
            .copy(foo = ""), // A comment
        other = ""
    )
}