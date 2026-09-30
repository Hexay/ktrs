fun foo() =
    FooBar
        .fooBar(
            "foobar"
        ).filter {
            it == "bar"
        }.associate { ruleSetProviderV1 ->
            "bar"
        }.also {
            bar()
        }