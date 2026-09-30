val foo1 = "Foo = $foo"

@Suppress("RemoveCurlyBracesFromTemplate")
val foo2 = "Foo = ${foo}"

@Suppress("RemoveCurlyBracesFromTemplate", "OtherSuppression")
val foo3 = "Foo = ${foo}"