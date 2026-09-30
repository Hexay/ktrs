val foo =
    when (bar) {
        BAR1 -> { "bar1" }
        BAR2 -> // some comment 1
            // some comment 2
            "bar2"
        else -> null
    }