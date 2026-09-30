val foo = Array(2) { 42 }
val bar1 = foo[1]
val bar2 = foo[
    1 // The comma before the comment should be removed without removing the comment itself
]
val bar3 = foo[
    1 /* The comma before the comment should be removed without removing the comment itself */
]