val foo1 = "foo1"
val foo2: String
    get() = "foo2"
var foo3: String = "foo3"
    set(value) {
        field = value.repeat(2)
    }
var foo4 = "foo4"
var foo5: String by Delegate()