abstract class A(init: String.() -> Int)
class B : A({
    toInt()
})

fun test(a: Any, b: (Any) -> Any) {
    test(a = "1", b = {
        it.toString()
    })
}