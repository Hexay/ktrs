interface Foo

class Bar(a: Int, b: Int, c: Int) : Foo

class Test3 :
    Foo by Bar(
        a = 1,
        b = 2,
        c = 3
    )