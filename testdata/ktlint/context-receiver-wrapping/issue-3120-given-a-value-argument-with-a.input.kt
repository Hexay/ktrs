class Foo(
  foo: context(Int, Int) (Int) -> Unit,
) {
    constructor(
      bar: String,
      foo: context(Int, Int) (Int) -> Unit,
    ) : this(foo)
}

fun foo(
  foo: context(Int, Int) (Int) -> Unit,
)

val foo1: () -> Unit
    set(value: context(MyContext) () -> Unit) {
        // some-code
    }

val foo2 = { param: context(MyContext) () -> Unit ->
    // some-code
}