enum class Foo {
    A,
    @Bar1 B,
    @Bar1 @Bar2 C,
    @Bar3("bar3") D,
    @Bar4("bar4") E,
    @Bar3("bar3") @Bar1 F,
    @Bar4("bar4") @Bar1 G
}