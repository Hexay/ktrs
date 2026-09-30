private fun test(): Boolean? =
    runCatching { true }
        .getOrNull()?.let { result ->
            !result
        }