annotation class Bar(val description: String, val names: Array<String>)

@Bar(
description = "foo",
names = arrayOf(
"foo" +
"bar"
)
)
private val foo: String = "foo"