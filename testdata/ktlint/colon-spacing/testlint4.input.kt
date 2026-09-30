class A {
    @Deprecated("") @Throws(IOException::class, SecurityException::class)
    protected abstract fun <T> f(
        @Nullable thing: String, things: List<T>): Runnable where T : Runnable, T: Closeable
}