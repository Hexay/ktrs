// The code example below not make sense. Its importance is that modifier list can contain multiple elements
fun foo(
    vararg
    noinline
    crossinline
    a: Any
) = "some-result"