val foo = Array(2) { 42 }
val bar1 = foo[1]
val bar2 = foo[
    1, // The comma should be inserted before the comment
]
val bar3 = foo[
    1, /* The comma should be inserted before the comment */
]