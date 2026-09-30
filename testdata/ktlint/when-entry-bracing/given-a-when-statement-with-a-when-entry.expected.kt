val foo =
    when (bar) {
        BAR1 -> {
            "bar1" // bar 1 comment
        }
        // comment before BAR2
        BAR2 -> {
            "bar2" // bar 2 comment
        }
        else -> {
            null // else comment
        }
    }