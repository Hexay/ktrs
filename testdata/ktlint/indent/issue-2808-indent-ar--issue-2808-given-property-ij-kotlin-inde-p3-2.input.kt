val foo =
    when (bar()) {
        is Bar1
        -> "bar1"

        is Bar2
        -> "bar2"
            .also { println(it) }

        else
        ->
            "bar3"
                .also { println(it) }
    }