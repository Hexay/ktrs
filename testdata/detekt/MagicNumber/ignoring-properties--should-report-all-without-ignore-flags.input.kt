@Magic(number = 69)
class A {
    val boringNumber = 42
    const val BORING_CONSTANT = 93871
    val duration = Duration.seconds(10)
    val durationWithStdlibFunction = 10.toDuration(DurationUnit.MILLISECONDS)

    override fun hashCode(): Int {
        val iAmSoMagic = 7328672
    }

    companion object {
        val anotherBoringNumber = 43
        const val anotherBoringConstant = 93872
        val color = Color(0x66000000)
        val colorWithExplicitParameter = Color(color = 0x66000000)
    }
}

data class Color(val color: Int)