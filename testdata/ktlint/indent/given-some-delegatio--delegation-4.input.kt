interface Foo

class Bar(a: Int, b: Int, c: Int) : Foo

class Test4 :
    Foo
    by Bar(
        a = 1,
        b = 2,
        c = 3
    )