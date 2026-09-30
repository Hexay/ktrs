annotation class Annotation(val params: IntArray)

@Annotation([1, 2,])
val foo1: Int = 0

@Annotation([
    1,
    2, // The comma before the comment should be removed without removing the comment itself
])
val foo2: Int = 0

@Annotation([
    1,
    2, /* The comma before the comment should be removed without removing the comment itself */
])
val foo3: Int = 0