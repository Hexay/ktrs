fun f() {
    try {
    } catch (e: IllegalStateException) {
        if (true) {
            val message = e.message
            throw IllegalArgumentException(message)
        }
    } catch (f: Exception) {
        val message = f.toString()
        if (true) {
            throw Exception(IllegalArgumentException(message))
        }
    }
}