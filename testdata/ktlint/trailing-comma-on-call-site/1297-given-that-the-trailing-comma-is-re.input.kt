annotation class FooBar(
    val foo1: Array<String> = [],
    val foo2: Array<String> = [],
    val bar1: String = ""
)

@FooBar(
    foo1 = [
        "foo-1" // Add trailing comma as the argument value list foo1 is a multiline statement
    ],
    foo2 = ["foo-2"], // Do not add trailing comma in the array as the argument value list foo2 is a single line statement
    bar1 = "bar-1" // Add trailing comma as the outer argument value list of the annotation is a multiline statement
)
val fooBar = null