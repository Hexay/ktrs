val foo =
    listOf("foo")
        .let { bar ->
            if (fooBar > 42) {
                "foo"
            } else {
                "bar"
            }
        }