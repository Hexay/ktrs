val fooBar1: (Int, Int) -> Int = { foo, bar -> foo * bar }
val fooBar2: (Int, Int) -> Int = {
    foo,
    bar, // The comma should be inserted before the comment
    ->
    foo * bar
}
val fooBar3: (Int, Int) -> Int = {
    foo,
    bar, /* The comma should be inserted before the comment */
    ->
    foo * bar
}