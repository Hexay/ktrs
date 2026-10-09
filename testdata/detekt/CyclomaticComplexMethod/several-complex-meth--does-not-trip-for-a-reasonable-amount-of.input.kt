fun f() {
    val map = HashMap<Any, String>()
    for ((key, value) in map) {
        when (key) {
            is Int -> print("int")
            is String -> print("String")
            is Float -> print("Float")
            is Double -> print("Double")
            is Byte -> print("Byte")
            is Short -> print("Short")
            is Long -> print("Long")
            is Boolean -> print("Boolean")
            else -> throw IllegalArgumentException("Unexpected type value")
        }
    }
}