fun f() {
    try {
    } catch (e: IllegalArgumentException) {
        print(e)
    } catch (e: Exception) {
        print(e.message)
    }
}