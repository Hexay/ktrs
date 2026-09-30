class SampleClass {

    private val public = "ok" // ok
    private val private = "not_ok" // false positive
    private val halfPublicHalfPrivate = "not_ok" // false positive

    /**
     * Doc 1
     */
    fun one() = 1

    /**
     * Doc 2
     */
    fun two() = 2
    fun three() {
        val public = "ok" // ok
        val private = "not_ok" // false positive
    }
}

enum class SampleEnum {
    One, // ok
    Two, // false positive
    Three, // false positive
}