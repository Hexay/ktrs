annotation class Annotation(val params: IntArray)

@Annotation([1, 2])
val foo1: Int = 0

@Annotation([
    1,
    2 // The comma should be inserted before the comment
])
val foo2: Int = 0

@Annotation([
    1,
    2 /* The comma should be inserted before the comment */
])
val foo3: Int = 0

@Annotation(
    [
        1,
        2 /* The comma should be inserted before the comment */
    ],
    [
        3,
        4 /* The comma should be inserted before the comment */
    ]
)
val foo4: Int = 0