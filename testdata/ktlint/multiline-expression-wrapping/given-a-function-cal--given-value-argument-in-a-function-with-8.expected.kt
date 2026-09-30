val foo =
    foo(
        "The quick brown fox "
            .takeIf { it.jumps }
            ?.plus("jumps over the lazy dog"),
    )