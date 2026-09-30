val foo1: (String, String) -> String = {
        s1: String,
        s2: String
    ->
    // does something with strings
}

val foo2 = {
        s1: String,
        // Trailing comma on last element is allowed and does not have effect
        s2: String,
    ->
    // does something with strings
}