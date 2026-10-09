open class A {
    var data: ByteArray = ByteArray(0)
    var data2: ByteArray = ByteArray(0)
    var data3: ByteArray = ByteArray(0)
}

class B: A() {
    fun test(): ByteArray? {
        val data = data
        if (data.isEmpty()) return null
        if (data.contains(-1)) return null
        if (data.contains(-2)) return null
        if (data.contains(-3)) return null
        val data2 = data2
        if (data2.isEmpty()) return null
        if (data2.contains(-1)) return null
        if (data2.contains(-2)) return null
        if (data2.contains(-3)) return null
        val data3 = data3
        if (data3.isEmpty()) return null
        if (data3.contains(-1)) return null
        if (data3.contains(-2)) return null
        if (data3.contains(-3)) return null
        return data

    }
}