fun foo(): Boolean {
    return true &&
        columns.all { col ->
            false
        }
}