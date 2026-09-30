// Max line length marker:         #
val foo1 = foobar(foo * bar) + "foo"
val foo2 = foobar(foo * bar) + "fooo"
val foo3 = foobar("fooooooo", bar) + "foo"
val foo4 = foobar("foooooooo", bar) + "foo"
val foo5 = foobar("fooooooooo", bar) + "foo"
val foo6 = foobar("foooo", foo * bar) + "foo"
val foo7 = foobar("fooooooooooo", foo * bar) + "foo"