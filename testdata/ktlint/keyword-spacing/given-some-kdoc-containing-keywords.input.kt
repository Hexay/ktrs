/**
 * Convenience wrapper around [Mockito.when] to avoid special `when` notation.
 */
fun <T> whenever(call: T): OngoingStubbing<T> = Mockito.`when`(call)