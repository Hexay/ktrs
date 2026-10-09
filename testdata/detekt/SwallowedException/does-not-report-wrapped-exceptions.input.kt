fun f() {
    try {
    } catch (e: IllegalStateException) {
        throw IllegalArgumentException(e.message, e)
    } catch (e: Exception) {
        throw IllegalArgumentException(e)
    }
}