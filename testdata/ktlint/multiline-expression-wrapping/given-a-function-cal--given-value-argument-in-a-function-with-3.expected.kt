val foo =
    foo(
        parameterName =
            "The quick brown fox "
                .takeIf { it.jumps }
                ?.plus("jumps over the lazy dog"),
    )