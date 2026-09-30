abstract class A(init: String.() -> Int)
class B : A({
    toInt() // This line exceeds the line limit but will not be reported
})

val foo1 =
    test(a = "1", b = {
        it.toString() // This line exceeds the line limit but will not be reported
    })
val foo2 =
    test(a = "1")
val foo3 =
    test(
        a = "1",
        b = "2"
    )