interface Foo

class Bar(a: Int, b: Int, c: Int) : Foo

class Test5 {
    companion object : Foo by Bar(
        a = 1,
        b = 2,
        c = 3
    )
}