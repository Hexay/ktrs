val foo =
    when (bar) {
        BAR1 -> {
            "bar1"
        }
        BAR2 -> {
            "bar2"
                .plus("bar3")
                .plus("bar4")
        }
        else -> {
            null
        }
    }