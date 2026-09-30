interface Foo

class Bar(a: Int, b: Int, c: Int) : Foo

class Test4 :
    Foo
    by Bar(
        a = 1,
        b = 2,
        c = 3
    )

// The next line ensures that the fix regarding the expectedIndex due to alignment of "by" keyword in
// class above, is still in place. Without this fix, the expectedIndex would hold a negative value,
// resulting in the formatting to crash on the next line.
val bar = 1