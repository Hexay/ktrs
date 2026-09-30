interface Printable {
    fun print(): String
}
enum class Shape : Printable {
    Square {
        override fun print() = "■"
    },
    Triangle {
        override fun print() = "▲"
    },
}