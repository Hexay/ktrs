fun foo(
    a: Int,
    @Bar1 b: Int,
    @Bar1 @Bar2 c: Int,
    @Bar3("bar3") @Bar1 d: Int
) {}