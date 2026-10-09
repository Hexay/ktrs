fun f() {
    try {
    } catch (e: IllegalStateException) {
        val message = e.message
        throw IllegalArgumentException(message)
    } catch (f: Exception) {
        val message = f.toString()
        throw Exception(IllegalArgumentException(message))
    }
}