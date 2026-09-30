val string =
    listOf("The quick brown fox", "jumps", "over the lazy dog")
        .map {
            it
                .lowercase()
                .substringAfter("o")
        }