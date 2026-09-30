fun bar() {
    Pair("val1", "val2")
        .let { (first, second) ->
                first + second
        }
}