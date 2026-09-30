fun foo(): Boolean {
    return true // some comment
            &&
        columns.all { col ->
            false
        }
}