fun f() {
    try {
    } catch (e: IllegalStateException) {
        throw IllegalArgumentException(e.message)
    } catch (f: Exception) {
        throw Exception(IllegalArgumentException(f.toString()))
    }
}