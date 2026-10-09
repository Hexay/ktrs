fun f(condition: Boolean) {
    try {
        println()
    } catch (e: IllegalStateException) {
        if (condition) {
            throw IllegalArgumentException(e.message)
        }
        throw IllegalArgumentException(e)
    }
}