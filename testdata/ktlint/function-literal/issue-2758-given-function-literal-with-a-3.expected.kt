val foo =
    when {
        false -> { { "bar" } }
        else -> { { "baz" } }
    }