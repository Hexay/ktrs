fun foo1() = "Foo = $foo"

@Suppress("RemoveCurlyBracesFromTemplate")
fun foo2() = "Foo = ${foo}"

@Suppress("RemoveCurlyBracesFromTemplate", "OtherSuppression")
fun foo3() = "Foo = ${foo}"