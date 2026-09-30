// Max line length marker:                                #
val foo1 = "foo".filter { it.isUpperCase() }.lowercase() // Some comment
val foo2 =
    "foo"
        .filter { it.isUpperCase() } // Some longgggggggg comment
        .lowercase()