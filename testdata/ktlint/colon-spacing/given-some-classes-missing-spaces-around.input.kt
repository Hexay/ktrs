open class C1(string: String)
class C2(string: String):C1(string)
class C3 constructor(string: String):C1(string)
class C4:C1 {
    constructor():this("")
    constructor(param: String):super(param)

    val c1 = object:C1("foo") { }
    val c2 = C4()
}