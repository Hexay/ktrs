val foo1: (String) -> String = { // Some comment which should not be moved to the next line when formatting
        s: String
    ->
    // does something with string
}

val foo2: (String) -> String = {
        // Some comment which has to be indented with the parameter list
        s: String
    ->
    // does something with string
}

val foo3 = { // Some comment which should not be moved to the next line when formatting
        s: String,
    ->
    // does something with string
}

val foo4 = {
        // Some comment which has to be indented with the parameter list
        s: String,
    ->
    // does something with string
}