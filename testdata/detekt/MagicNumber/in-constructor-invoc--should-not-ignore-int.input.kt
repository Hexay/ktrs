data class Model(
        val someVal: Int,
        val other: String = "default"
)

var model = Model(someVal = 53)