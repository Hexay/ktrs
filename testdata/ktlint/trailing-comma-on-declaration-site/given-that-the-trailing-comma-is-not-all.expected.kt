val fooBar1: (Int, Int) -> Int = { foo, bar -> foo * bar }
val fooBar2: (Int, Int) -> Int = {
    foo,
    bar // The comma before the comment should be removed without removing the comment itself
    ->
    foo * bar
}
val fooBar3: (Int, Int) -> Int = {
    foo,
    bar /* The comma before the comment should be removed without removing the comment itself */
    ->
    foo * bar
}