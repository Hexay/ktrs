fun f(condition: Boolean) {
    try {
        println()
    } catch (e: IllegalStateException) {
        try {
        } catch (nested: Exception) {
            throw IllegalArgumentException()
        }
        throw IllegalArgumentException(e)
    }
}