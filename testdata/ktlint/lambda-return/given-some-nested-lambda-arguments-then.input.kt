val foo =
    "Foo"
        .let outer@{ foo ->
            if (foo != "Foo") {
                return@outer "$foo is not expected input"
            }
            foo
                .let { return@let "$it was a" }
                .let secondLet@{
                    if (foo != "Foo") {
                        return@secondLet "$foo is not expected input"
                    }
                    return@secondLet "$it success"
                }.let { return@outer "$it (map after let)" }
            // This is unreachable code due to "return@outer" in let above.
            return@outer "$foo was a failure"
        }.let { return@let "$it (let after also)" }