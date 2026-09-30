fun test(
    x: Int,
    y: Int,
    block: (Int, Int) -> Int
): (
    Int, Int
) -> Int = { foo, bar ->
    block(foo, bar)
}