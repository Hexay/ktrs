// https://github.com/ktlint/ktlint/issues/764#issuecomment-646822853
val foo1 = println({
    bar()
}, {
    bar()
})
// Other formats which should be allowed as well
val foo2 = println(
    {
        bar()
    },
    { bar() }
)
val foo3 = println(
    // Some comment
    {
        bar()
    },
    // Some comment
    { bar() }
)
val foo4 = println(
    /* Some comment */
    {
        bar()
    },
    /* Some comment */
    { bar() }
)
val foo5 = println(
    { bar() },
    { bar() }
)
val foo6 = println(
    // Some comment
    { bar() },
    // Some comment
    { bar() }
)
val foo7 = println(
    /* Some comment */
    { bar() },
    /* Some comment */
    { bar() }
)
val foo8 = println(
    { bar() }, { bar() }
)
val foo9 = println({ bar() }, { bar()})