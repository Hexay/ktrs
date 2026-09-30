val foo =
    "Foo"
        .let outer@{ foo ->
            if (foo != "Foo") {
                return@outer "$foo is not expected input"
            }
            foo
                .let { "$it was a" }
                .let secondLet@{
                    if (foo != "Foo") {
                        return@secondLet "$foo is not expected input"
                    }
                    "$it success"
                }.let { return@outer "$it (map after let)" }
            // This is unreachable code due to "return@outer" in let above.
            "$foo was a failure"
        }.let { "$it (let after also)" }