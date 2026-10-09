open class A {
    var data: ByteArray = ByteArray(0)
}

class B: A() {
    fun test(): ByteArray? {
        val data1 = data
        if (data1.isEmpty()) return null
        if (data1.contains(-1)) return null
        if (data1.contains(-2)) return null
        if (data1.contains(-3)) return null
        return data1

    }
}