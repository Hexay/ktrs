fun f() {
    try {
    } catch (ignore: IllegalArgumentException) {
    } catch (expected: Exception) {
    }
}