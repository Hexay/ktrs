public fun Window.asCoroutineDispatcher(): CoroutineDispatcher =
    @Suppress("UnsafeCastFromDynamic")
    asDynamic().coroutineDispatcher ?: WindowDispatcher(this).also {
        asDynamic().coroutineDispatcher = it
    }