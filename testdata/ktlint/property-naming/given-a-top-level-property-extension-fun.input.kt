val fooBar1: Any
    get() = foobar() // Lint can not check whether data is immutable
val fooBar2
    inline get() = foobar() // Lint can not check whether data is immutable
val fooBar3
    @Bar get() = foobar() // Lint can not check whether data is immutable