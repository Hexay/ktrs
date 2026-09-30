val foo =
    when (value) {
        0 -> { foo(); true }
        else -> { bar(); false }
    }