open class A {
    var data: ByteArray = ByteArray(0)
}

class B: A() {
    fun test(): ByteArray? {
        val data = data
        if (data.isEmpty()) return null
        if (data.contains(-1)) return null
        if (data.contains(-2)) return null
        if (data.contains(-3)) return null
        return data

    }
}