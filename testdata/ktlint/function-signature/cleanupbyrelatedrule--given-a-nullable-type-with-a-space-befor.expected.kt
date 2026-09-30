fun String?.f1() = "some-result"
fun List<String?>.f2() = "some-result"
fun f3(string: String?) = "some-result"
fun f4(string: List<String?>) = "some-result"
fun f5(): String? = "some-result"
fun f6(): List<String?> = listOf("some-result", null)
fun f7(): List<String>? = null