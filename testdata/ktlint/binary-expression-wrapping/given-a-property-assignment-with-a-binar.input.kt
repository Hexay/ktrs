// Max line length marker:         #
val foo1 = foobar(foo * bar) + "foo"
val foo2 = foobar("foo", foo * bar) + "foo"
val foo3 = foobar("fooo", foo * bar) + "foo"
@Suppress("ktlint:standard:max-line-length")
val foo4 = foobar("fooo", foo * bar) + "foo"