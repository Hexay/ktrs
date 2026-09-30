// Max line length marker:                  #
val foo2 =
    "foo"
        .filter {
             it
                 .uppercase()  // Some comment
                 .isUpperCase()
        }.lowercase()